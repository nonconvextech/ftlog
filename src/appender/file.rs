use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use time::{Duration, PreciseTime, Tm};

#[derive(Clone, Copy)]
pub enum Period {
    Minute,
    Hour,
    Day,
    Month,
    Year,
}
struct Rotate {
    start: PreciseTime,
    wait: Duration,
    period: Period,
    keep: Option<std::time::Duration>,
}

pub struct FileAppender {
    file: BufWriter<File>,
    path: PathBuf,
    rotate: Option<Rotate>,
}
impl FileAppender {
    /// Create a file appender that write log to file
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        let p = path.as_ref();
        Self {
            file: BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .unwrap(),
            ),
            path: p.to_path_buf(),
            rotate: None,
        }
    }

    fn file<T: AsRef<Path>>(path: T, period: Period) -> PathBuf {
        let p = path.as_ref();
        let tm = time::now();
        let ts = match period {
            Period::Year => {
                let year = tm.tm_year + 1900;
                format!("{}", year)
            }
            Period::Month => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                format!("{}{:02}", year, month)
            }
            Period::Day => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                let day = tm.tm_mday;
                format!("{}{:02}{:02}", year, month, day)
            }
            Period::Hour => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                let day = tm.tm_mday;
                let hour = tm.tm_hour;
                format!("{}{:02}{:02}T{:02}", year, month, day, hour)
            }
            Period::Minute => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                let day = tm.tm_mday;
                let hour = tm.tm_hour;
                let minute = tm.tm_min;
                format!("{}{:02}{:02}T{:02}{:02}", year, month, day, hour, minute)
            }
        };

        if let Some(ext) = p.extension() {
            let file_name = p
                .file_stem()
                .map(|x| format!("{}-{}.{}", x.to_string_lossy(), ts, ext.to_string_lossy()))
                .expect("invalid file name");
            p.with_file_name(file_name)
        } else {
            p.with_file_name(format!(
                "{}-{}",
                p.file_name()
                    .map(|x| x.to_string_lossy())
                    .unwrap_or(Cow::from("log")),
                ts
            ))
        }
    }

    /// Create a file appender that rotate a new file every given period
    pub fn rotate<T: AsRef<Path>>(path: T, period: Period) -> Self {
        let p = path.as_ref();
        let (start, wait) = Self::until(period);
        let path = Self::file(&p, period);
        let file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .unwrap(),
        );
        Self {
            file,
            path: p.to_path_buf(),
            rotate: Some(Rotate {
                start,
                wait,
                period,
                keep: None,
            }),
        }
    }

    /// Create a file appender with rotate, auto delete logs that last modified
    /// before given expire duration
    pub fn rotate_with_expire<T: AsRef<Path>>(path: T, period: Period, keep: Duration) -> Self {
        let p = path.as_ref();
        let (start, wait) = Self::until(period);
        let path = Self::file(&p, period);
        let file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .unwrap(),
        );
        Self {
            file,
            path: p.to_path_buf(),
            rotate: Some(Rotate {
                start,
                wait,
                period,
                keep: Some(keep.to_std().unwrap()),
            }),
        }
    }

    fn until(period: Period) -> (PreciseTime, Duration) {
        let tm_now = time::now();
        let now = PreciseTime::now();
        let tm_next = Self::next(&tm_now, period);
        (now, tm_next - tm_now)
    }

    #[inline]
    fn next(now: &Tm, period: Period) -> Tm {
        let mut tm_next = now.clone();
        let tm_next = match period {
            Period::Year => {
                tm_next.tm_mon = 0;
                tm_next.tm_mday = 1;
                tm_next.tm_hour = 0;
                tm_next.tm_min = 0;
                tm_next.tm_sec = 0;
                tm_next.tm_nsec = 0;
                tm_next.tm_year += 1;
                time::at(tm_next.to_timespec())
            }
            Period::Month => {
                tm_next.tm_mon += 1;
                tm_next.tm_mday = 1;
                tm_next.tm_hour = 0;
                tm_next.tm_min = 0;
                tm_next.tm_sec = 0;
                tm_next.tm_nsec = 0;
                time::at(tm_next.to_timespec())
            }
            Period::Day => {
                tm_next.tm_mday += 1;
                tm_next.tm_hour = 0;
                tm_next.tm_min = 0;
                tm_next.tm_sec = 0;
                tm_next.tm_nsec = 0;
                time::at(tm_next.to_timespec())
            }
            Period::Hour => {
                tm_next.tm_hour += 1;
                tm_next.tm_min = 0;
                tm_next.tm_sec = 0;
                tm_next.tm_nsec = 0;
                time::at(tm_next.to_timespec())
            }
            Period::Minute => {
                tm_next.tm_min += 1;
                tm_next.tm_sec = 0;
                tm_next.tm_nsec = 0;
                time::at(tm_next.to_timespec())
            }
        };
        tm_next
    }
}

impl Write for FileAppender {
    fn write(&mut self, record: &[u8]) -> std::io::Result<usize> {
        if let Some(Rotate {
            start,
            wait,
            period,
            keep,
        }) = &mut self.rotate
        {
            if start.to(PreciseTime::now()) > *wait {
                // close current file and create new file
                self.file.flush()?;
                let path = Self::file(&self.path, *period);
                // remove outdated log files
                if let Some(keep_duration) = keep {
                    let to_remove = std::fs::read_dir(self.path.parent().unwrap())
                        .unwrap()
                        .filter_map(|f| f.ok())
                        .filter(|x| x.file_type().map(|x| x.is_file()).unwrap_or(false))
                        .filter(|x| {
                            let p = x.path();
                            let name = p.file_stem().unwrap().to_string_lossy();
                            if let Some((stem, time)) = name.rsplit_once("-") {
                                let check = |(ix, x): (usize, char)| match ix {
                                    8 => x == 'T',
                                    _ => x.is_digit(10),
                                };
                                let len = match period {
                                    Period::Minute => time.len() == 13,
                                    Period::Hour => time.len() == 11,
                                    Period::Day => time.len() == 8,
                                    Period::Month => time.len() == 6,
                                    Period::Year => time.len() == 4,
                                };
                                len && time.chars().enumerate().all(check)
                                    && self
                                        .path
                                        .file_stem()
                                        .map(|x| x.to_string_lossy() == stem)
                                        .unwrap_or(false)
                            } else {
                                false
                            }
                        })
                        .filter(|x| {
                            x.metadata()
                                .ok()
                                .and_then(|x| x.modified().ok())
                                .map(|time| {
                                    time.elapsed()
                                        .map(|elapsed| elapsed > *keep_duration)
                                        .unwrap_or(false)
                                })
                                .unwrap_or(false)
                        });

                    let del_msg = to_remove
                        .filter(|f| std::fs::remove_file(f.path()).is_ok())
                        .map(|x| x.file_name().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if !del_msg.is_empty() {
                        crate::info!("Log file deleted: {}", del_msg);
                    }
                };

                // rotate file
                self.file = BufWriter::new(
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                        .unwrap(),
                );
                (*start, *wait) = Self::until(*period);
            }
        };
        self.file.write_all(record).map(|_| record.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use time::Timespec;
    #[test]
    fn to_wait_ms() {
        // Mon Oct 24 2022 16:00:00 GMT+0000
        let now = time::at(Timespec::new(1666627200, 0)).to_utc();

        let tm_next = FileAppender::next(&now, Period::Year);
        let tm = time::at(Timespec::new(1672531200, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileAppender::next(&now, Period::Month);
        let tm = time::at(Timespec::new(1667260800, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileAppender::next(&now, Period::Day);
        let tm = time::at(Timespec::new(1666656000, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileAppender::next(&now, Period::Hour);
        let tm = time::at(Timespec::new(1666630800, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileAppender::next(&now, Period::Minute);
        let tm = time::at(Timespec::new(1666627260, 0));
        println!("{}", tm_next.to_timespec().sec);
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());
    }
}
