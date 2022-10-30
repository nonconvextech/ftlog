use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::Write,
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

pub struct FileSplit {
    file: File,
    path: PathBuf,

    start: PreciseTime,
    wait: Duration,

    period: Period,
}
impl FileSplit {
    // TODO custom format
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
                format!("{}{}", year, month)
            }
            Period::Day => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                let day = tm.tm_mday;
                format!("{}{}{}", year, month, day)
            }
            Period::Hour => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                let day = tm.tm_mday;
                let hour = tm.tm_hour;
                format!("{}{}{}T{}", year, month, day, hour)
            }
            Period::Minute => {
                let year = tm.tm_year + 1900;
                let month = tm.tm_mon + 1;
                let day = tm.tm_mday;
                let hour = tm.tm_hour;
                let minute = tm.tm_min;
                format!("{}{}{}T{}{}", year, month, day, hour, minute)
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
    pub fn new<T: AsRef<Path>>(path: T, period: Period) -> Self {
        let p = path.as_ref();
        let (start, wait) = Self::until(period);
        let path = Self::file(&p, period);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        Self {
            file,
            path: p.to_path_buf(),
            start,
            wait,
            period,
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

impl Write for FileSplit {
    fn write(&mut self, record: &[u8]) -> std::io::Result<usize> {
        if self.start.to(PreciseTime::now()) > self.wait {
            // close current file and create new file
            self.file.flush()?;
            let path = Self::file(&self.path, self.period);
            self.file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            (self.start, self.wait) = Self::until(self.period);
        }
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

        let tm_next = FileSplit::next(&now, Period::Year);
        let tm = time::at(Timespec::new(1672531200, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileSplit::next(&now, Period::Month);
        let tm = time::at(Timespec::new(1667260800, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileSplit::next(&now, Period::Day);
        let tm = time::at(Timespec::new(1666656000, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileSplit::next(&now, Period::Hour);
        let tm = time::at(Timespec::new(1666630800, 0));
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());

        let tm_next = FileSplit::next(&now, Period::Minute);
        let tm = time::at(Timespec::new(1666627260, 0));
        println!("{}", tm_next.to_timespec().sec);
        assert_eq!(tm_next, tm, "{} != {}", now.rfc3339(), tm_next.rfc3339());
    }
}
