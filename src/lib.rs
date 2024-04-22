//! [![Build Status](https://github.com/nonconvextech/ftlog/workflows/ftlog/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
//! ![License](https://img.shields.io/crates/l/ftlog.svg)
//! [![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
//! [![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)
//!
//! Logging is affected by the disk IO and pipe system call.
//! Sequential log calls can be a bottleneck in scenarios where low
//! latency is critical (e.g., high-frequency trading).
//!
//! `ftlog` mitigates this bottleneck by sending messages to a dedicated logger
//! thread and computing as little as possible in the main/worker thread.
//!
//! `ftlog` can improve log performance in main/worker thread a few times over. See
//! performance for details.
//!
//!
//! # Usage
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! ftlog = "0.2"
//! ```
//!
//! Configure and initialize ftlog at the start of your `main` function:
//! ```
//! // ftlog re-export `log`'s macros, so no need to add `log` to dependencies
//! use ftlog::appender::FileAppender;
//! use ftlog::{debug, trace};
//! use log::{error, info, warn};
//!
//! // minimal configuration with default setting
//!
//! // When drops, the guard calls and waits `flush` to logger.
//! // With guard that share the lifetime of `main` fn, there is no need to manually call flush at the end of `main` fn.
//! let _guard = ftlog::builder().try_init().unwrap();
//!
//! trace!("Hello world!");
//! debug!("Hello world!");
//! info!("Hello world!");
//! warn!("Hello world!");
//! error!("Hello world!");
//! ```
//!
//! A more complicated but feature rich usage:
//!
//! ```rust,no_run
//! use ftlog::{
//!     appender::{Duration, FileAppender, Period},
//!     FtLogFormatter, LevelFilter,
//! };
//!
//! let time_format = time::format_description::parse_owned::<1>(
//!     "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]",
//! )
//! .unwrap();
//! // configurate logger
//! let _guard = ftlog::builder()
//!     // global max log level
//!     .max_log_level(LevelFilter::Info)
//!     // custom timestamp format
//!     .time_format(time_format)
//!     // set global log formatter
//!     .format(FtLogFormatter)
//!     // use bounded channel to avoid large memory comsumption when overwhelmed with logs
//!     // Set `false` to tell ftlog to discard excessive logs.
//!     // Set `true` to block log call to wait for log thread.
//!     // here is the default settings
//!     .bounded(100_000, false) // .unbounded()
//!     // define root appender, pass anything that is Write and Send
//!     // omit `Builder::root` will write to stderr
//!     .root(
//!         FileAppender::builder()
//!             .path("./current.log")
//!             .rotate(Period::Day)
//!             .expire(Duration::days(7))
//!             .build(),
//!     )
//!     // timezone of log message timestamp, use local by default
//!     .local_timezone()
//!     // or use fiexed timezone for better throughput, since retrieving timezone is a time consuming operation
//!     // this does not affect worker threads (that call log), but can boost log thread performance (higher throughput).
//!     .fixed_timezone(time::UtcOffset::current_local_offset().unwrap())
//!     // level filter for root appender
//!     .root_log_level(LevelFilter::Warn)
//!     // write logs in ftlog::appender to "./ftlog-appender.log" instead of "./current.log"
//!     .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
//!     .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
//!     .try_init()
//!     .expect("logger build or set failed");
//! ```
//!
//! See `./examples` for more (e.g. custom format).
//!
//! ## Default Log Format
//!
//! > 2022-04-08 19:20:48.190+08 **298ms** INFO main [src/ftlog.rs:14] My log
//! > message
//!
//! Here `298ms` denotes the latency between the call of the log (e.g.
//! `log::info!("msg")`) and the actual printing in log thread. Normally this is 0ms.
//!
//! A large delay indicates that the log thread may be blocked by excessive log
//! messages.
//!
//! > 2022-04-10 21:27:15.996+08 0ms **2** INFO main [src/main.rs:29] limit
//! > running3 !
//!
//! The number **2** above indicates how many log messages were discarded.
//! Only shown if the frequency of logging for a single log call is limited (e.g.
//! `log::info!(limit=3000i64;"msg")`).
//!
//! ## Randomly drop log
//!
//! Use `random_drop` or `drop` to specify the probability of randomly discarding logs.
//! No message is dropped by default.
//!
//! ```rust
//! log::info!(random_drop=0.1f32;"Random log 10% of log calls, keeps 90%");
//! log::info!(drop=0.99f32;"Random drop 99% of log calls, keeps 1%");
//! ```
//!
//! This can be helpful when formatting log message into string is too costly,
//!
//! When both `random_drop` and `limit` is specified,
//! ftlog will limit logs after messages are randomly dropped.
//! ```rust
//! log::info!(drop=0.99f32,limit=1000;
//!     "Drop 99% messages. The survived 1% messages are limit to at least 1000ms between adjacent log messages output"
//! );
//! ```
//!
//! ## Custom timestamp format
//!
//! `ftlog` relies on the `time` crate for the formatting of timestamp. To use custom time format,
//! first construct a valid time format description,
//! and then pass it to ftlog builder by `ftlog::time_format(&mut self)`.
//!
//! In case an error occurs when formatting timestamp, `ftlog` will fallback to RFC3339 time format.
//!
//! ### Example
//! ```rust
//! let format = time::format_description::parse_owned::<1>(
//!     "[year]/[month]/[day] [hour]:[minute]:[second].[subsecond digits:6]",
//! )
//! .unwrap();
//! let _guard = ftlog::builder().time_format(format).try_init().unwrap();
//! log::info!("Log with custom timestamp format");
//! // Output:
//! // 2023/06/14 11:13:26.160840 0ms INFO main [main.rs:3] Log with custom timestamp format
//! ```
//!
//! ## Log with interval
//!
//! `ftlog` allows to limit the write frequency for individual log calls.
//!
//! If the above line is called multiple times within 3000ms, then it is logged only
//! once, with an added number reflecting the number of discarded log messages.
//!
//! Each log call ha an independent interval, so we can set different intervals
//! for different log calls. Internally, `ftlog` records the last print time by a
//! combination of (module name, file name, code line).
//!
//! ### Example
//!
//! ```rust
//! # use ftlog::info;
//! info!(limit=3000i64; "limit running {}s !", 3);
//! ```
//! The minimal interval of the the specific log call above is 3000ms.
//!
//! ```markdown
//! 2022-04-10 21:27:10.996+08 0ms 0 INFO main [src/main.rs:29] limit running 3s !
//! 2022-04-10 21:27:15.996+08 0ms 2 INFO main [src/main.rs:29] limit running 3s !
//! ```
//! The number **2** above shows how many log messages is discarded since last log.
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
//! Log rotation is configured in `FileAppender`, and the timestamp is appended to
//! the end of the filename:
//!
//! ```rust
//! use ftlog::appender::{FileAppender, Period};
//!
//! let logger = ftlog::builder()
//!     .root(
//!         FileAppender::builder()
//!             .path("./mylog.log")
//!             .rotate(Period::Minute)
//!             .build(),
//!     )
//!     .build()
//!     .unwrap();
//! let _guard = logger.init().unwrap();
//! ```
//!
//! If the log file is configured to be split by minutes,
//! the log file name has the format
//! `mylog-{MMMM}{YY}{DD}T{hh}{mm}.log`. When divided by days, the log file name is
//! something like `mylog-{MMMM}{YY}{DD}.log`.
//!
//! Log filename examples:
//! ```sh
//! $ ls
//! # by minute
//! current-20221026T1351.log
//! # by hour
//! current-20221026T13.log
//! # by day
//! current-20221026.log
//! # by month
//! current-202210.log
//! # by year
//! current-2022.log
//! # omitting extension (e.g. "./log") will add datetime to the end of log filename
//! log-20221026T1351
//! ```
//!
//! ### Clean outdated logs
//!
//! With log rotation enabled, it is possible to clean outdated logs to free up
//! disk space with `FileAppender::rotate_with_expire` method or set `expire(Duration)`
//! when using builder.
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
//!
//! // Rotate every day, clean stale logs that were modified 7 days ago on each rotation
//! let appender = FileAppender::rotate_with_expire("./current.log", Period::Day, Duration::days(7));
//! let logger = ftlog::builder()
//!     .root(appender)
//!     .build()
//!     .unwrap();
//! let _guard = logger.init().unwrap();
//! ```
//!
//! # Features
//! - **tsc**
//!   Use [TSC](https://en.wikipedia.org/wiki/Time_Stamp_Counter) for clock source for higher performance without
//!   accuracy loss.
//!   
//!   TSC offers the most accurate and cheapest way to access current time under certain condition:
//!   1. the CPU frequency must be constant
//!   1. must with CPU of x86/x86_64 architecture, since TSC is an x86/x86_64 specific register.
//!   1. never suspend
//!
//!   The current feature further requires that the build target **MUST BE LINUX**. Otherwise it will fall back to
//!   a fast but much less accurate implementation.
//!   
//! # Timezone
//!
//! For performance, timezone is detected once at logger buildup, and use it later in every
//! log message. This is partly due to timezone detetion is expensive, and partly to the unsafe
//! nature of underlying system call in multi-thread program in Linux.
//!
//! It's also recommended to use UTC instead to further avoid timestamp convertion to timezone for every log message.
//!
//! # Performance
//!
//! > Rust：1.67.0-nightly
//!
//! |                                                   |  message type | Apple M1 Pro, 3.2GHz  | AMD EPYC 7T83, 3.2GHz |
//! | ------------------------------------------------- | ------------- | --------------------- | --------------------- |
//! | `ftlog`                                           | static string |   75 ns/iter    | 385 ns/iter    |
//! | `ftlog`                                           | with i32      |   106 ns/iter   | 491 ns/iter    |
//! | `env_logger` <br/> output to file                 | static string | 1,674 ns/iter  | 1,142 ns/iter   |
//! | `env_logger` <br/> output to file                 | with i32      | 1,681 ns/iter   | 1,179 ns/iter   |
//! | `env_logger` <br/> output to file with `BufWriter`| static string | 279 ns/iter     | 550 ns/iter     |
//! | `env_logger` <br/> output to file with `BufWriter`| with i32      | 278 ns/iter     | 565 ns/iter     |

use arc_swap::ArcSwap;
pub use log::{
    debug, error, info, log, log_enabled, logger, trace, warn, Level, LevelFilter, Record,
};
use time::format_description::OwnedFormatItem;
use time::{OffsetDateTime, UtcOffset};

use std::borrow::Cow;
use std::fmt::Display;
use std::hash::{BuildHasher, Hash, Hasher};
use std::io::{stderr, Error as IoError, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, unbounded, Receiver, RecvTimeoutError, Sender, TrySendError};
use hashbrown::HashMap;
use log::{kv::Key, set_boxed_logger, set_max_level, Log, Metadata, SetLoggerError};

pub mod appender;

use tm::{duration, now, to_utc, Time};

#[cfg(not(feature = "tsc"))]
mod tm {
    use super::*;

    pub type Time = std::time::SystemTime;
    #[inline]
    pub fn now() -> Time {
        std::time::SystemTime::now()
    }
    #[inline]
    pub fn to_utc(time: Time) -> OffsetDateTime {
        time.into()
    }

    #[inline]
    pub fn duration(from: Time, to: Time) -> Duration {
        to.duration_since(from).unwrap_or_default()
    }
}

#[cfg(feature = "tsc")]
mod tm {
    use super::*;
    pub type Time = minstant::Instant;
    #[inline]
    pub fn now() -> Time {
        minstant::Instant::now()
    }
    #[inline]
    pub fn to_utc(time: Time) -> OffsetDateTime {
        static ANCHOR: once_cell::sync::Lazy<minstant::Anchor> =
            once_cell::sync::Lazy::new(|| minstant::Anchor::new());
        OffsetDateTime::from_unix_timestamp_nanos(time.as_unix_nanos(&ANCHOR) as i128).unwrap()
    }
    #[inline]
    pub fn duration(from: Time, to: Time) -> Duration {
        to.duration_since(from)
    }
}

#[cfg(target_family = "unix")]
fn local_timezone() -> UtcOffset {
    UtcOffset::current_local_offset().unwrap_or_else(|_| {
        let tz = tz::TimeZone::local().unwrap();
        let current_local_time_type = tz.find_current_local_time_type().unwrap();
        let diff_secs = current_local_time_type.ut_offset();
        UtcOffset::from_whole_seconds(diff_secs).unwrap()
    })
}
#[cfg(not(target_family = "unix"))]
fn local_timezone() -> UtcOffset {
    UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)
}

struct LogMsg {
    time: Time,
    msg: Box<dyn Sync + Send + Display>,
    level: Level,
    target: String,
    limit: u32,
    limit_key: u64,
}
impl LogMsg {
    fn write(
        self,
        filters: &Vec<Directive>,
        appenders: &mut HashMap<&'static str, Box<dyn Write + Send>>,
        root: &mut Box<dyn Write + Send>,
        root_level: LevelFilter,
        missed_log: &mut HashMap<u64, i64, nohash_hasher::BuildNoHashHasher<u64>>,
        last_log: &mut HashMap<u64, Time, nohash_hasher::BuildNoHashHasher<u64>>,
        offset: Option<UtcOffset>,
        time_format: &time::format_description::OwnedFormatItem,
    ) {
        let msg = self.msg.to_string();
        if msg.is_empty() {
            return;
        }

        let now = now();

        let writer = if let Some(filter) = filters.iter().find(|x| self.target.starts_with(x.path))
        {
            if filter.level.map(|l| l < self.level).unwrap_or(false) {
                return;
            }
            filter
                .appender
                .and_then(|n| appenders.get_mut(n))
                .unwrap_or(root)
        } else {
            if root_level < self.level {
                return;
            }
            root
        };

        if self.limit > 0 {
            let missed_entry = missed_log.entry(self.limit_key).or_insert_with(|| 0);
            if let Some(last) = last_log.get(&self.limit_key) {
                if duration(*last, now) < Duration::from_millis(self.limit as u64) {
                    *missed_entry += 1;
                    return;
                }
            }
            last_log.insert(self.limit_key, now);
            let delay = duration(self.time, now);
            let utc_datetime = to_utc(self.time);

            let offset_datetime = offset
                .map(|o| utc_datetime.to_offset(o))
                .unwrap_or(utc_datetime);

            let s = format!(
                "{} {}ms {} {}\n",
                offset_datetime
                    .format(&time_format)
                    .unwrap_or_else(|_| offset_datetime
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap()),
                delay.as_millis(),
                *missed_entry,
                msg
            );
            if let Err(e) = writer.write_all(s.as_bytes()) {
                eprintln!("logger write message failed: {}", e);
            };
            *missed_entry = 0;
        } else {
            let delay = duration(self.time, now);
            let utc_datetime = to_utc(self.time);
            let offset_datetime = offset
                .map(|o| utc_datetime.to_offset(o))
                .unwrap_or(utc_datetime);
            let s = format!(
                "{} {}ms {}\n",
                offset_datetime
                    .format(&time_format)
                    .unwrap_or_else(|_| offset_datetime
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap()),
                delay.as_millis(),
                msg
            );
            if let Err(e) = writer.write_all(s.as_bytes()) {
                eprintln!("logger write message failed: {}", e);
            };
        }
    }
}

enum LoggerInput {
    LogMsg(LogMsg),
    Flush,
}

#[derive(Debug)]
enum LoggerOutput {
    Flushed,
    FlushError(std::io::Error),
}

/// Shared by ftlog formatter
///
/// To further reduce time spent on log macro calls, ftlog saves required data
/// and later construct log string in log thread.
///
/// `FtLogFormat` defines how to turn an reference to record into a box object,
/// which can be sent to log thread and later formatted into string.
///
/// Here is an example of custom formatter:
///
/// ```
/// use std::fmt::Display;
///
/// use ftlog::FtLogFormat;
/// use log::{Level, Record};
///
/// struct MyFormatter;
/// impl FtLogFormat for MyFormatter {
///     fn msg(&self, record: &Record) -> Box<dyn Send + Sync + Display> {
///         Box::new(Msg {
///             level: record.level(),
///             thread: std::thread::current().name().map(|n| n.to_string()),
///             file: record.file_static(),
///             line: record.line(),
///             args: format!("{}", record.args()),
///             module_path: record.module_path_static(),
///         })
///     }
/// }
/// // Store necessary field, define how to format into string with `Display` trait.
/// struct Msg {
///     level: Level,
///     thread: Option<String>,
///     file: Option<&'static str>,
///     line: Option<u32>,
///     args: String,
///     module_path: Option<&'static str>,
/// }
///
/// impl Display for Msg {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.write_str(&format!(
///             "{}@{}||{}:{}[{}] {}",
///             self.thread.as_ref().map(|x| x.as_str()).unwrap_or(""),
///             self.module_path.unwrap_or(""),
///             self.file.unwrap_or(""),
///             self.line.unwrap_or(0),
///             self.level,
///             self.args
///         ))
///     }
/// }
/// ```
pub trait FtLogFormat: Send + Sync {
    /// turn an reference to record into a box object, which can be sent to log thread
    /// and then formatted into string.
    fn msg(&self, record: &Record) -> Box<dyn Send + Sync + Display>;
}

/// Default ftlog formatter
///
/// The default ftlog format is like:
/// ```text
/// INFO main [examples/ftlog.rs:27] Hello, world!
/// ```
///
/// Since ftlog cannot customize timestamp, the corresponding part is omitted.
/// The actual log output is like:
/// ```text
/// 2022-11-22 17:02:12.574+08 0ms INFO main [examples/ftlog.rs:27] Hello, world!
/// ```
pub struct FtLogFormatter;
impl FtLogFormat for FtLogFormatter {
    /// Return a box object that contains required data (e.g. thread name, line of code, etc.) for later formatting into string
    #[inline]
    fn msg(&self, record: &Record) -> Box<dyn Send + Sync + Display> {
        Box::new(Message {
            level: record.level(),
            thread: std::thread::current().name().map(|n| n.to_string()),
            file: record
                .file_static()
                .map(|s| Cow::Borrowed(s))
                .or_else(|| record.file().map(|s| Cow::Owned(s.to_owned())))
                .unwrap_or(Cow::Borrowed("")),
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
    file: Cow<'static, str>,
    line: Option<u32>,
    args: Cow<'static, str>,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} {} [{}:{}] {}",
            self.level,
            self.thread.as_ref().map(|x| x.as_str()).unwrap_or(""),
            self.file,
            self.line.unwrap_or(0),
            self.args
        ))
    }
}

struct DiscardState {
    last: ArcSwap<Instant>,
    count: AtomicUsize,
}

/// A guard that flushes logs associated to a Logger on a drop
///
/// With this guard, you can ensure all logs are written to destination
/// when the application exits.
pub struct LoggerGuard {
    queue: Sender<LoggerInput>,
    notification: Receiver<LoggerOutput>,
}
impl Drop for LoggerGuard {
    fn drop(&mut self) {
        self.queue
            .send(LoggerInput::Flush)
            .expect("logger queue closed when flushing, this is a bug");
        self.notification
            .recv()
            .expect("logger notification closed, this is a bug");
    }
}
/// ftlog global logger
pub struct Logger {
    format: Box<dyn FtLogFormat>,
    level: LevelFilter,
    queue: Sender<LoggerInput>,
    notification: Receiver<LoggerOutput>,
    block: bool,
    discard_state: Option<DiscardState>,
    stopped: AtomicBool,
}

impl Logger {
    pub fn init(self) -> Result<LoggerGuard, SetLoggerError> {
        let guard = LoggerGuard {
            queue: self.queue.clone(),
            notification: self.notification.clone(),
        };

        set_max_level(self.level);
        let boxed = Box::new(self);
        set_boxed_logger(boxed).map(|_| guard)
    }
}

impl Log for Logger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        // already checked in log macros
        self.level >= metadata.level()
    }

    fn log(&self, record: &Record) {
        #[cfg(feature = "random_drop")]
        {
            let random_drop = record
                .key_values()
                .get(Key::from_str("random_drop"))
                .or_else(|| record.key_values().get(Key::from_str("drop")))
                .and_then(|x| x.to_f64())
                .unwrap_or(1.) as f32;
            if random_drop < 1. && fastrand::f32() < random_drop {
                return;
            }
        }

        let limit = record
            .key_values()
            .get(Key::from_str("limit"))
            .and_then(|x| x.to_u64())
            .unwrap_or(0) as u32;

        let msg = self.format.msg(record);
        let limit_key = if limit == 0 {
            0
        } else {
            let mut b = hashbrown::hash_map::DefaultHashBuilder::default().build_hasher();
            if let Some(p) = record.module_path() {
                p.as_bytes().hash(&mut b);
            } else {
                record.file().unwrap_or("").as_bytes().hash(&mut b);
            }
            record.line().unwrap_or(0).hash(&mut b);
            b.finish()
        };
        let msg = LoggerInput::LogMsg(LogMsg {
            time: now(),
            msg: msg,
            target: record.target().to_owned(),
            level: record.level(),
            limit,
            limit_key,
        });
        if self.block {
            if let Err(_) = self.queue.send(msg) {
                let stop = self.stopped.load(Ordering::SeqCst);
                if !stop {
                    eprintln!("logger queue closed when logging, this is a bug");
                    self.stopped.store(true, Ordering::SeqCst)
                }
            }
        } else {
            match self.queue.try_send(msg) {
                Err(TrySendError::Full(_)) => {
                    if let Some(s) = &self.discard_state {
                        let count = s.count.fetch_add(1, Ordering::SeqCst);
                        if s.last.load().elapsed().as_secs() >= 5 {
                            eprintln!("Excessive log messages. Log omitted: {}", count);
                            s.last.store(Arc::new(Instant::now()));
                        }
                    }
                }
                Err(TrySendError::Disconnected(_)) => {
                    let stop = self.stopped.load(Ordering::SeqCst);
                    if !stop {
                        eprintln!("logger queue closed when logging, this is a bug");
                        self.stopped.store(true, Ordering::SeqCst)
                    }
                }
                _ => (),
            }
        }
    }

    fn flush(&self) {
        self.queue
            .send(LoggerInput::Flush)
            .expect("logger queue closed when flushing, this is a bug");
        if let LoggerOutput::FlushError(err) = self
            .notification
            .recv()
            .expect("logger notification closed, this is a bug")
        {
            eprintln!("Fail to flush: {}", err);
        }
    }
}

struct BoundedChannelOption {
    size: usize,
    block: bool,
    print: bool,
}

/// Ftlog builder
///
/// ```
/// # use ftlog::appender::{FileAppender, Duration, Period};
/// # use log::LevelFilter;
/// let logger = ftlog::builder()
///     // use our own format
///     .format(ftlog::FtLogFormatter)
///     // global max log level
///     .max_log_level(LevelFilter::Info)
///     // define root appender, pass anything that is Write and Send
///     // omit `Builder::root` to write to stderr
///     .root(FileAppender::rotate_with_expire(
///         "./current.log",
///         Period::Day,
///         Duration::days(7),
///     ))
///     // ---------- configure additional filter ----------
///     // write to "ftlog-appender" appender, with different level filter
///     .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
///     // write to root appender, but with different level filter
///     .filter("ftlog", None, LevelFilter::Trace)
///     // write to "ftlog" appender, with default level filter
///     .filter("ftlog::appender::file", "ftlog", None)
///     // ----------  configure additional appender ----------
///     // new appender
///     .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
///     // new appender, rotate to new file every Day
///     .appender("ftlog", FileAppender::rotate("ftlog.log", Period::Day))
///     .build()
///     .expect("logger build failed");
/// ```
///
/// # Local timezone
/// For performance reason, `ftlog` only retrieve timezone info once and use this
/// local timezone offset forever. Thus timestamp in log does not aware of timezone
/// change by OS.
pub struct Builder {
    format: Box<dyn FtLogFormat>,
    time_format: Option<OwnedFormatItem>,
    level: Option<LevelFilter>,
    root_level: Option<LevelFilter>,
    root: Box<dyn Write + Send>,
    appenders: HashMap<&'static str, Box<dyn Write + Send + 'static>>,
    filters: Vec<Directive>,
    bounded_channel_option: Option<BoundedChannelOption>,
    timezone: LogTimezone,
}

/// Handy function to get ftlog builder
#[inline]
pub fn builder() -> Builder {
    Builder::new()
}

struct Directive {
    path: &'static str,
    level: Option<LevelFilter>,
    appender: Option<&'static str>,
}
/// timezone for log
pub enum LogTimezone {
    /// local timezone
    ///
    /// Only *unix OS is supported for now
    Local,
    /// UTC timezone
    Utc,
    /// fixed timezone
    Fixed(UtcOffset),
}

impl Builder {
    #[inline]
    /// Create a ftlog builder with default settings:
    /// - global log level: INFO
    /// - root log level: INFO
    /// - default formatter: `FtLogFormatter`
    /// - output to stderr
    /// - bounded channel between worker thread and log thread, with a size limit of 100_000
    /// - discard excessive log messages
    /// - log with timestamp of local timezone
    pub fn new() -> Builder {
        Builder {
            format: Box::new(FtLogFormatter),
            level: None,
            root_level: None,
            root: Box::new(stderr()) as Box<dyn Write + Send>,
            appenders: HashMap::new(),
            filters: Vec::new(),
            bounded_channel_option: Some(BoundedChannelOption {
                size: 100_000,
                block: false,
                print: true,
            }),
            timezone: LogTimezone::Local,
            time_format: None,
        }
    }

    /// Set custom formatter
    #[inline]
    pub fn format<F: FtLogFormat + 'static>(mut self, format: F) -> Builder {
        self.format = Box::new(format);
        self
    }

    /// Set custom datetime formatter
    #[inline]
    pub fn time_format(mut self, format: OwnedFormatItem) -> Builder {
        self.time_format = Some(format);
        self
    }

    /// bound channel between worker thread and log thread
    ///
    /// When `block_when_full` is true, it will block current thread where
    /// log macro (e.g. `log::info`) is called until log thread is able to handle new message.
    /// Otherwises, excessive log messages will be discarded.
    ///
    /// By default, excessive log messages is discarded silently. To show how many log
    /// messages have been dropped, see `Builder::print_omitted_count()`.
    #[inline]
    pub fn bounded(mut self, size: usize, block_when_full: bool) -> Builder {
        self.bounded_channel_option = Some(BoundedChannelOption {
            size,
            block: block_when_full,
            print: false,
        });
        self
    }

    /// whether to print the number of omitted logs if channel to log
    /// thread is bounded, and set to discard excessive log messages
    #[inline]
    pub fn print_omitted_count(mut self, print: bool) -> Builder {
        self.bounded_channel_option
            .as_mut()
            .map(|o| o.print = print);
        self
    }

    /// set channel size to unbound
    ///
    /// **ATTENTION**: too much log message will lead to huge memory consumption,
    /// as log messages are queued to be handled by log thread.
    /// When log message exceed the current channel size, it will double the size by default,
    /// Since channel expansion asks for memory allocation, log calls can be slow down.
    #[inline]
    pub fn unbounded(mut self) -> Builder {
        self.bounded_channel_option = None;
        self
    }

    /// Add an additional appender with a name
    ///
    /// Combine with `Builder::filter()`, ftlog can output log in different module
    /// path to different output target.
    #[inline]
    pub fn appender(
        mut self,
        name: &'static str,
        appender: impl Write + Send + 'static,
    ) -> Builder {
        self.appenders.insert(name, Box::new(appender));
        self
    }

    /// Add a filter to redirect log to different output
    /// target (e.g. stderr, stdout, different files).
    ///
    /// **ATTENTION**: level more verbose than `Builder::max_log_level` will be ignored.
    /// Say we configure `max_log_level` to INFO, and even if filter's level is set to DEBUG,
    /// ftlog will still log up to INFO.
    #[inline]
    pub fn filter<A: Into<Option<&'static str>>, L: Into<Option<LevelFilter>>>(
        mut self,
        module_path: &'static str,
        appender: A,
        level: L,
    ) -> Builder {
        let appender = appender.into();
        let level = level.into();
        if appender.is_some() || level.is_some() {
            self.filters.push(Directive {
                path: module_path,
                appender: appender,
                level: level,
            });
        }
        self
    }

    #[inline]
    /// Configure the default log output target.
    ///
    /// Omit this method will output to stderr.
    pub fn root(mut self, writer: impl Write + Send + 'static) -> Builder {
        self.root = Box::new(writer);
        self
    }

    #[inline]
    /// Set max log level
    ///
    /// Logs with level more verbose than this will not be sent to log thread.
    pub fn max_log_level(mut self, level: LevelFilter) -> Builder {
        self.level = Some(level);
        self
    }

    #[inline]
    /// Set max log level
    ///
    /// Logs with level more verbose than this will not be sent to log thread.
    pub fn root_log_level(mut self, level: LevelFilter) -> Builder {
        self.root_level = Some(level);
        self
    }

    #[inline]
    /// Log with timestamp of local timezone
    ///
    /// Timezone is fixed after logger setup for the following reasons:
    /// 1. `time` v0.3 currently do not allow access to local offset for multithread process
    /// in unix-like OS.
    /// 1. timezone retrieval from OS is quite slow (around several microsecond) compare with
    /// utc timestamp retrieval (around tens of nanoseconds)
    pub fn local_timezone(mut self) -> Builder {
        self.timezone = LogTimezone::Local;
        self
    }

    #[inline]
    /// Log with timestamp of UTC timezone
    pub fn utc(mut self) -> Builder {
        self.timezone = LogTimezone::Utc;
        self
    }

    #[inline]
    /// Log with timestamp of fixed timezone
    pub fn fixed_timezone(mut self, timezone: UtcOffset) -> Builder {
        self.timezone = LogTimezone::Fixed(timezone);
        self
    }

    #[inline]
    /// Specify the timezone of log messages
    pub fn timezone(mut self, timezone: LogTimezone) -> Builder {
        self.timezone = timezone;
        self
    }

    /// Finish building ftlog logger
    ///
    /// The call spawns a log thread to formatting log message into string,
    /// and write to output target.
    pub fn build(self) -> Result<Logger, IoError> {
        let offset = match self.timezone {
            LogTimezone::Local => Some(local_timezone()),
            LogTimezone::Utc => None,
            LogTimezone::Fixed(offset) => Some(offset),
        };
        let time_format = self.time_format.unwrap_or_else(|| {
            time::format_description::parse_owned::<1>(
                "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]+[offset_hour]",
            )
            .unwrap()
        });
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
        let global_level = self.level.unwrap_or(LevelFilter::Info);
        let root_level = self.root_level.unwrap_or(global_level);
        if global_level < root_level {
            warn!(
                "Logs with level more verbose than {} will be ignored",
                global_level,
            );
        }

        let (sync_sender, receiver) = match &self.bounded_channel_option {
            None => unbounded(),
            Some(option) => bounded(option.size),
        };
        let (notification_sender, notification_receiver) = bounded(1);
        std::thread::Builder::new()
            .name("logger".to_string())
            .spawn(move || {
                let mut appenders = self.appenders;
                let filters = filters;

                for filter in &filters {
                    if let Some(level) = filter.level {
                        if global_level < level {
                            warn!(
                                "Logs with level more verbose than {} will be ignored in `{}` ",
                                global_level, filter.path,
                            );
                        }
                    }
                }

                let mut root = self.root;
                let mut last_log = HashMap::default();
                let mut missed_log = HashMap::default();
                let mut last_flush = Instant::now();
                let timeout = Duration::from_millis(200);
                loop {
                    match receiver.recv_timeout(timeout) {
                        Ok(LoggerInput::LogMsg(log_msg)) => {
                            log_msg.write(
                                &filters,
                                &mut appenders,
                                &mut root,
                                root_level,
                                &mut missed_log,
                                &mut last_log,
                                offset,
                                &time_format,
                            );
                        }
                        Ok(LoggerInput::Flush) => {
                            let max = receiver.len();
                            'queue: for _ in 1..=max {
                                if let Ok(LoggerInput::LogMsg(msg)) = receiver.try_recv() {
                                    msg.write(
                                        &filters,
                                        &mut appenders,
                                        &mut root,
                                        root_level,
                                        &mut missed_log,
                                        &mut last_log,
                                        offset,
                                        &time_format,
                                    )
                                } else {
                                    break 'queue;
                                }
                            }
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
                        Err(RecvTimeoutError::Timeout) => {
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
                        }
                        Err(e) => {
                            eprintln!(
                                "sender closed without sending a Quit first, this is a bug, {}",
                                e
                            );
                        }
                    }
                }
            })?;
        let block = self
            .bounded_channel_option
            .as_ref()
            .map(|x| x.block)
            .unwrap_or(false);
        let print = self
            .bounded_channel_option
            .as_ref()
            .map(|x| x.print)
            .unwrap_or(false);
        Ok(Logger {
            format: self.format,
            level: global_level,
            queue: sync_sender,
            notification: notification_receiver,
            block,
            discard_state: if block || !print {
                None
            } else {
                Some(DiscardState {
                    last: ArcSwap::new(Arc::new(Instant::now())),
                    count: AtomicUsize::new(0),
                })
            },
            stopped: AtomicBool::new(false),
        })
    }

    /// try building and setting as global logger
    pub fn try_init(self) -> Result<LoggerGuard, Box<dyn std::error::Error>> {
        let logger = self.build()?;
        Ok(logger.init()?)
    }
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Builder::new()
    }
}
