#![feature(unchecked_math)]
//! [![Build Status](https://github.com/nonconvextech/ftlog/workflows/CI%20%28Linux%29/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
//! ![License](https://img.shields.io/crates/l/ftlog.svg)
//! [![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
//! [![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)
//!
//! Logging is affected by disk IO and pipe system call.
//! Sequencial log call can a bottleneck in scenario where low
//! latency is critical (e.g. high frequency trading).
//!
//! `ftlog` alleviates this bootleneck by sending message to dedicated logger
//! thread, and computing as little as possible in main/worker thread.
//!
//! `ftlog` can boost log performance by **times** in main/worker thread. See
//! performance for details.
//!
//! **CAUTION**: this crate use `unchecked_math` unstable feature and `unsafe`
//! code. Only use this crate in rust `nightly` channel.
//!
//!
//! # Usage
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! ftlog = "0.2.0"
//! ```
//!
//! Configure and initialize ftlog at the start of your `main` function:
//! ```rust
//! // ftlog re-export `log`'s macros, so no need to add `log` to dependencies
//! use ftlog::{debug, trace};
//! use log::{error, info, warn};
//!
//! fn main() {
//!     // minimal configuration with default setting
//!     // define root appender, pass None would write to stderr
//!     let dest = FileAppender::new("./current.log");
//!     ftlog::builder().root(dest).build().unwrap().init().unwrap();
//!
//!     trace!("Hello world!");
//!     debug!("Hello world!");
//!     info!("Hello world!");
//!     warn!("Hello world!");
//!     error!("Hello world!");
//!
//!     // when main thread is done, logging thread may be busy printing messages
//!     // wait for log output to flush, otherwise messages in memory yet might lost
//!     ftlog::logger().flush();
//! }
//! ```
//!
//! A more complicated but feature rich usage:
//!
//! ```rust,no_run
//! use ftlog::{appender::{Period, FileAppender}, LevelFilter, Record, FtLogFormat};
//!
//! // Custom formatter
//! // A formatter defines how to build a message.
//! // Since Formatting message into string can slow down the log macro call,
//! // the idomatic way is to send required field as is to log thread, and build message in log thread.
//! struct MyFormatter;
//! impl FtLogFormat for MyFormatter {
//!     fn msg(&self, record: &Record) -> Box<dyn Send + Sync + std::fmt::Display> {
//!         Box::new(Msg {
//!             level: record.level(),
//!             thread: std::thread::current().name().map(|n| n.to_string()),
//!             file: record.file_static(),
//!             line: record.line(),
//!             args: format!("{}", record.args()),
//!             module_path: record.module_path_static(),
//!         })
//!     }
//! }
//!
//! // Store necessary field, define how to build into string with `Display` trait.
//! struct Msg {
//!     level: Level,
//!     thread: Option<String>,
//!     file: Option<&'static str>,
//!     line: Option<u32>,
//!     args: String,
//!     module_path: Option<&'static str>,
//! }
//!
//! impl Display for Msg {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         f.write_str(&format!(
//!             "{}@{}||{}:{}[{}] {}",
//!             self.thread.as_ref().map(|x| x.as_str()).unwrap_or(""),
//!             self.module_path.unwrap_or(""),
//!             self.file.unwrap_or(""),
//!             self.line.unwrap_or(0),
//!             self.level,
//!             self.args
//!         ))
//!     }
//! }
//!
//! // configurate logger
//! let logger = ftlog::builder()
//!     // use custom fotmat
//!     // datetime format is not configurable for performance
//!     .format(StringFormatter)
//!     // global max log level
//!     .max_log_level(LevelFilter::Info)
//!     // define root appender, pass None would write to stderr
//!     .root(FileAppender::rotate_with_expire(
//!         "./current.log",
//!         Period::Minute,
//!         Duration::seconds(30),
//!     ))
//!     // ---------- configure additional filter ----------
//!     // write to "ftlog-appender" appender, with different level filter
//!     .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
//!     // write to root appender, but with different level filter
//!     .filter("ftlog", None, LevelFilter::Trace)
//!     // write to "ftlog" appender, with default level filter
//!     .filter("ftlog::appender::file", "ftlog", None)
//!     // ----------  configure additional appender ----------
//!     // new appender
//!     .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
//!     // new appender, rotate to new file every Day
//!     .appender("ftlog", FileAppender::rotate("ftlog.log", Period::Day))
//!     .build()
//!     .expect("logger build failed");
//! // init global logger
//! logger.init().expect("set logger failed");
//! ```
//!
//! See `examples/complex.rs`.
//!
//! ## Default Log Format
//!
//! Datetime format is fixed for performance.
//!
//! > 2022-04-08 19:20:48.190+08 **298ms** INFO main/src/ftlog.rs:14 My log
//! > message
//!
//! Here `298ms` denotes the latency between log macros call (e.g.
//! `log::info!("msg")`) and actual printing in log thread. Normally this will
//! be 0ms.
//!
//! A large delay indicates log thread might be blocked by excessive log
//! messages.
//!
//! > 2022-04-10 21:27:15.996+08 0ms **2** INFO main/src/main.rs:29 limit
//! > running3 !
//!
//! The number **2** above shows how many log messages have been discarded.
//! Only shown when limiting logging interval for a single log call (e.g.
//! `log::info!(limit=3000;"msg")`).
//!
//! ## Log with interval
//!
//! `ftlog` allows restriction on writing frequency for single log call.
//!
//! If the above line is called multiple times within 3000ms, then it will only
//! log once with a added number donates the number of discarded log message.
//!
//! Each log call owns independent interval, so we can set different interval
//! for different log call. Internally, `ftlog` records last print time by a
//! combination of (module name, file name, line of code).
//!
//! ### Example
//!
//! ```rust
//! # use ftlog::info;
//! info!(limit=3000; "limit running{} !", 3);
//! info!(limit=1000; "limit running{} !", 1);
//! ```
//! The minimal interval of the the above log call is 3000ms.
//!
//! ```plain
//! 2022-04-10 21:27:15.996+08 0ms 2 INFO main/src/main.rs:29 limit running1 !
//! ```
//! The number **2** above shows how many log messages have been discarded.
//!
//! ## Log rotation
//! `ftlog` supports log rotation in local timezone. The available rotation
//! periods are:
//!
//! - minute `Period::Minute`
//! - hour `Period::Hour`
//! - day `Period::Day`
//! - month `Period::Month`
//! - year `Period::Year`
//!
//! Log rotation is configured in `FileAppender`, and datetime will be added to
//! the end of filename:
//!
//! ```rust
//! use ftlog::appender::{FileAppender, Period};
//!
//! let logger = ftlog::builder()
//!     .root(FileAppender::rotate("./mylog.log", Period::Minute))
//!     .build()
//!     .unwrap();
//! logger.init().unwrap();
//! ```
//!
//! When configured to divide log file by minutes,
//! the file name of log file is in the format of
//! `mylog-{MMMM}{YY}{DD}T{hh}{mm}.log`. When by days, the log file names is
//! something like `mylog-{MMMM}{YY}{DD}.log`.
//!
//! Log filename examples:
//! ```sh
//! $ ls
//! # by minute
//! current-20221026T1351.log
//! current-20221026T1352.log
//! current-20221026T1353.log
//! # by hour
//! current-20221026T13.log
//! current-20221026T14.log
//! # by day
//! current-20221026.log
//! current-20221027.log
//! # by month
//! current-202210.log
//! current-202211.log
//! # by year
//! current-2022.log
//! current-2023.log
//!
//! # omitting extension (e.g. "./log") will add datetime to the end of log filename
//! log-20221026T1351
//! log-20221026T1352
//! log-20221026T1353
//! ```
//!
//! ### Clean outdated logs
//!
//! With log rotation enabled, it is possible to clean outdated logs to free up
//! disk space with `FileAppender::rotate_with_expire` method.
//!
//! `ftlog` first finds files generated by `ftlog` and cleans outdated logs by
//! last modified time. `ftlog` find generated logs by filename matched by file
//! stem and added datetime.
//!
//! **ATTENTION**: Any files that matchs the pattern will be deleted.
//!
//! ```rust
//! use ftlog::{appender::{Period, FileAppender, Duration}};
//!
//! // clean files named like `current-\d{8}T\d{4}.log`.
//! // files like `another-\d{8}T\d{4}.log` or `current-\d{8}T\d{4}` will not be deleted, since the filenames' stem do not match.
//! // files like `current-\d{8}.log` will remains either, since the rotation durations do not match.
//! let appender = FileAppender::rotate_with_expire("./current.log", Period::Minute, Duration::seconds(180));
//! let logger = ftlog::builder()
//!     .root(appender)
//!     .build()
//!     .unwrap();
//! logger.init().unwrap();
//! ```
//!
//! # Performance
//!
//! |  | Apple M1 Pro, 3.2GHz, Rust：1.67.0-nightly |  |
//! | ----------| --------- | ---------- |
//! | `ftlog` output to file | 156 ns/iter (+/- 23) | 1,636 ns/iter (+/- 229) |
//! | `env_logger` output to file | 1,636 ns/iter (+/- 229) | 1,636 ns/iter (+/- 229) |
//! | `ftlog` output to file with `BufWriter` | 154 ns/iter (+/- 23) | 274 ns/iter (+/- 41) |
//! | `env_logger` output to file with `BufWriter` | 274 ns/iter (+/- 41) | 607 ns/iter (+/- 41) |

pub use log::{
    debug, error, info, log, log_enabled, logger, trace, warn, Level, LevelFilter, Record,
};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{stderr, Error as IoError, Write};
use std::time::{Duration, Instant};

use crossbeam_channel::TryRecvError;
use fxhash::FxHashMap;
use log::kv::Key;
use log::{set_boxed_logger, set_max_level, Log, Metadata, SetLoggerError};
use time::{get_time, Timespec};

pub mod appender;

struct LogMsg {
    tm: Timespec,
    msg: Box<dyn Sync + Send + Display>,
    level: Level,
    target: String,
    limit: u32,
    limit_key: u64,
}

enum LoggerInput {
    LogMsg(LogMsg),
    Flush,
    Quit,
}

#[derive(Debug)]
enum LoggerOutput {
    Flushed,
    FlushError(std::io::Error),
}

pub trait FtLogFormat: Send + Sync {
    fn msg(&self, record: &Record) -> Box<dyn Send + Sync + Display>;
}
/// Default ftlog formatter
pub struct FtLogFormatter;
impl FtLogFormat for FtLogFormatter {
    fn msg(&self, record: &Record) -> Box<dyn Send + Sync + Display> {
        Box::new(Message {
            level: record.level(),
            thread: std::thread::current().name().map(|n| n.to_string()),
            file: record.file_static(),
            line: record.line(),
            args: record
                .args()
                .as_str()
                .map(|s| Cow::Borrowed(s))
                .unwrap_or_else(|| Cow::Owned(format!("{}", record.args()))),
        })
    }
}

struct Message {
    level: Level,
    thread: Option<String>,
    file: Option<&'static str>,
    line: Option<u32>,
    args: Cow<'static, str>,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} {}/{}:{} {}",
            self.level,
            self.thread.as_ref().map(|x| x.as_str()).unwrap_or(""),
            self.file.unwrap_or(""),
            self.line.unwrap_or(0),
            self.args
        ))
    }
}

pub struct Logger {
    format: Box<dyn FtLogFormat>,
    level: LevelFilter,
    queue: crossbeam_channel::Sender<LoggerInput>,
    notification: crossbeam_channel::Receiver<LoggerOutput>,
    worker_thread: Option<std::thread::JoinHandle<()>>,
}

impl Logger {
    pub fn init(self) -> Result<(), SetLoggerError> {
        set_max_level(self.level);
        let boxed = Box::new(self);
        set_boxed_logger(boxed)
    }
}

impl Log for Logger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        // already checked in log macros
        self.level >= metadata.level()
    }

    fn log(&self, record: &Record) {
        let limit = record
            .key_values()
            .get(Key::from_str("limit"))
            .and_then(|x| x.to_u64())
            .unwrap_or(0) as u32;
        let msg = self.format.msg(record);
        let limit_key = if limit == 0 {
            0
        } else {
            let mut hash_id = 2166136261_u64;
            unsafe {
                for c in record.module_path().unwrap_or("").as_bytes() {
                    hash_id = hash_id.unchecked_mul(16777619) ^ (*c as u64);
                }
                for c in record.file().unwrap_or("").as_bytes() {
                    hash_id = hash_id.unchecked_mul(16777619) ^ (*c as u64);
                }
                hash_id = hash_id.unchecked_mul(16777619) ^ (record.line().unwrap_or(0) as u64);
            }
            hash_id
        };
        self.queue
            .send(LoggerInput::LogMsg(LogMsg {
                tm: get_time(),
                msg: msg,
                target: record.target().to_owned(),
                level: record.level(),
                limit,
                limit_key,
            }))
            .expect("logger queue closed when logging, this is a bug")
    }

    fn flush(&self) {
        self.queue
            .send(LoggerInput::Flush)
            .expect("logger queue closed when flushing, this is a bug");
        self.notification
            .recv()
            .expect("logger notification closed, this is a bug");
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.queue
            .send(LoggerInput::Quit)
            .expect("logger queue closed before joining logger thread, this is a bug");
        let join_handle = self
            .worker_thread
            .take()
            .expect("logger thread empty when dropping logger, this is a bug");
        join_handle
            .join()
            .expect("failed to join logger thread when dropping logger, this is a bug");
    }
}

pub struct Builder {
    format: Box<dyn FtLogFormat>,
    level: LevelFilter,
    root: Option<Box<dyn Write + Send>>,
    appenders: HashMap<&'static str, Box<dyn Write + Send + 'static>>,
    filters: Vec<Directive>,
}

#[inline]
pub fn builder() -> Builder {
    Builder::new()
}

struct Directive {
    path: &'static str,
    level: Option<LevelFilter>,
    appender: Option<&'static str>,
}

impl Builder {
    #[inline]
    pub fn new() -> Builder {
        Builder {
            format: Box::new(FtLogFormatter),
            level: LevelFilter::Info,
            root: None,
            appenders: HashMap::new(),
            filters: Vec::new(),
        }
    }

    #[inline]
    pub fn format<F: FtLogFormat + 'static>(mut self, format: F) -> Builder {
        self.format = Box::new(format);
        self
    }

    #[inline]
    pub fn appender(
        mut self,
        name: &'static str,
        appender: impl Write + Send + 'static,
    ) -> Builder {
        self.appenders.insert(name, Box::new(appender));
        self
    }

    #[inline]
    pub fn filter<A: Into<Option<&'static str>>, L: Into<Option<LevelFilter>>>(
        mut self,
        target: &'static str,
        appender: A,
        level: L,
    ) -> Builder {
        let appender = appender.into();
        let level = level.into();
        if appender.is_some() || level.is_some() {
            self.filters.push(Directive {
                path: target,
                appender: appender,
                level: level,
            });
        }
        self
    }

    #[inline]
    pub fn root(mut self, writer: impl Write + Send + 'static) -> Builder {
        self.root = Some(Box::new(writer));
        self
    }

    #[inline]
    pub fn max_log_level(mut self, level: LevelFilter) -> Builder {
        self.level = level;
        self
    }

    pub fn build(self) -> Result<Logger, IoError> {
        let mut filters = self.filters;
        // sort filters' paths to ensure match for longest path
        filters.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
        filters.reverse();
        // check appender name in filters are all valid
        for appender_name in filters.iter().filter_map(|x| x.appender) {
            if !self.appenders.contains_key(appender_name) {
                panic!("Appender {} not configured", appender_name);
            }
        }

        let (sync_sender, receiver) = crossbeam_channel::unbounded();
        let (notification_sender, notification_receiver) = crossbeam_channel::bounded(1);
        let worker_thread = std::thread::Builder::new()
            .name("logger".to_string())
            .spawn(move || {
                let mut appenders = self.appenders;
                let filters = filters;

                for filter in &filters {
                    if let Some(level) = filter.level {
                        if self.level < level {
                            warn!(
                                "Logs with level more verbose than {} will be ignored in `{}` ",
                                self.level, filter.path,
                            );
                        }
                    }
                }

                let mut root: Box<dyn Write + Send> = match self.root {
                    Some(w) => w,
                    _ => Box::new(stderr()) as Box<dyn Write + Send>,
                };
                let mut last_log = FxHashMap::default();
                let mut missed_log = FxHashMap::default();
                let mut last_flush = Instant::now();
                loop {
                    match receiver.try_recv() {
                        Ok(LoggerInput::LogMsg(LogMsg {
                            tm,
                            msg,
                            limit,
                            limit_key,
                            target,
                            level,
                        })) => {
                            let now = get_time();
                            let now_nanos = now.sec * 1000_000_000 + now.nsec as i64;

                            let writer = if let Some(filter) =
                                filters.iter().find(|x| target.starts_with(x.path))
                            {
                                if filter.level.map(|l| l < level).unwrap_or(false) {
                                    continue;
                                }
                                filter
                                    .appender
                                    .map(|n| appenders.get_mut(n).unwrap())
                                    .unwrap_or(&mut root)
                            } else {
                                &mut root
                            };

                            if limit > 0 {
                                let missed_entry =
                                    missed_log.entry(limit_key).or_insert_with(|| 0_i64);
                                if let Some(last_nanos) = last_log.get(&limit_key) {
                                    if now_nanos - last_nanos < limit as i64 * 1_000_000 {
                                        *missed_entry += 1;
                                        continue;
                                    }
                                }
                                last_log.insert(limit_key, now_nanos);
                                let delay = (now_nanos - (tm.sec * 1000_000_000 + tm.nsec as i64))
                                    / 1_000_000;
                                let tm = time::at(tm);
                                let tm_millisec = tm.tm_nsec / 1_000_000;

                                writeln!(writer, "{:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3}{:>+03} {}ms {} {}",
                                         tm.tm_year + 1900,
                                         tm.tm_mon + 1,
                                         tm.tm_mday,
                                         tm.tm_hour,
                                         tm.tm_min,
                                         tm.tm_sec,
                                         tm_millisec,
                                         tm.tm_utcoff / 3600,
                                         delay,
                                         *missed_entry,
                                         msg
                                        ).expect("logger write message failed");
                                *missed_entry = 0;
                            } else {
                                let delay = (now_nanos - (tm.sec * 1000_000_000 + tm.nsec as i64))
                                    / 1_000_000;
                                let tm = time::at(tm);
                                let tm_millisec = tm.tm_nsec / 1_000_000;
                                writeln!(writer, "{:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3}{:>+03} {}ms {}",
                                         tm.tm_year + 1900,
                                         tm.tm_mon + 1,
                                         tm.tm_mday,
                                         tm.tm_hour,
                                         tm.tm_min,
                                         tm.tm_sec,
                                         tm_millisec,
                                         tm.tm_utcoff / 3600,
                                         delay,
                                         msg
                                        ).expect("logger write message failed");
                            }
                        }
                        Ok(LoggerInput::Flush) => {
                            let flush_result = appenders
                                .values_mut()
                                .chain([&mut root])
                                .find_map(|w| w.flush().err());
                            if let Some(error) = flush_result {
                                notification_sender
                                    .send(LoggerOutput::FlushError(error))
                                    .expect("logger notification failed");
                            } else {
                                notification_sender
                                    .send(LoggerOutput::Flushed)
                                    .expect("logger notification failed");
                            }
                        }
                        Ok(LoggerInput::Quit) => {
                            break;
                        }
                        Err(TryRecvError::Empty) => {
                            if last_flush.elapsed() > Duration::from_millis(1000) {
                                let flush_errors = appenders
                                    .values_mut()
                                    .chain([&mut root])
                                    .filter_map(|w| w.flush().err());
                                for err in flush_errors {
                                    log::warn!("Ftlog flush error: {}", err);
                                }
                                last_flush = Instant::now();
                            };
                            std::thread::sleep(Duration::from_micros(200));
                        }
                        Err(e) => {
                            panic!(
                                "sender closed without sending a Quit first, this is a bug, {}",
                                e
                            );
                        }
                    }
                }
            })?;
        Ok(Logger {
            format: self.format,
            level: self.level,
            queue: sync_sender,
            notification: notification_receiver,
            worker_thread: Some(worker_thread),
        })
    }
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Builder::new()
    }
}
