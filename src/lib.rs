#![feature(unchecked_math)]
//! 普通的日志库受到磁盘io和系统pipe影响，单线程顺序写入单条速度大概要2500ns（SSD），如果碰到io抖动或者慢磁盘，日志会是低延时交易的主要瓶颈。
//! 本库先把日志send到channel，再启动后台单独线程recv并且磁盘写入，测试速度在300ns左右。
//!
//! 本代码修改自`fastlog`以及官方的`log`库。
//!
//! CAUTION: this crate use `unchecked_math` unstable feature and `unsafe` code. Only use this crate in rust `nightly` channel.
//!
//! ### 日志格式
//! ```plain
//! 2022-04-08 19:20:48.190+08 298ms <<这里是日志的延时，正常这个数为0，如果太大说明channel堵塞了 INFO main/src/ftlog.rs:14 14575
//! ```
//!
//! ```plain
//! 2022-04-10 21:27:15.996+08 0ms 2 <<这表示丢弃了2条日志，说明使用了limit语法 INFO main/src/main.rs:29 limit running3 !
//! ```
//!
//! ## 配置
//! 加入引用
//! ```toml
//! ftlog = "0.1.0"
//! ```
//!
//! 在main开头加入配置：
//! ```rust
//! use ftlog::*;
//!
//! // 最简配置用法，使用默认配置，输出到stderr
//! LogBuilder::new().build().unwrap().init().unwrap();
//!
//! ```
//!
//! 默认的配置会把日志输出到stderr，程序启动的时候要执行bin/yourjob &>> logs/server.log来把日志定向到文件。（注意shell要用/bin/bash不要用/bin/sh。/bin/sh在ubuntu下有问题，不标准)
//!
//! 也可以直接输出到固定文件，完整的配置用法如下:
//!
//! ```rust
//! use ftlog::*;
//!
//! // 完整用法
//! // 配置logger
//! let logger = LogBuilder::new()
//!     //这里可以定义自己的格式，时间格式暂时不可以自定义
//!     .format(format)
//!     // a) 这里可以配置输出到文件
//!     .file(PathBuf::from("./current.log"))
//!     // b) 这里可以配置输出到文件，并且按指定间隔分割。这里导出的按天分割日志文件如current-20221024.log
//!     // 配置为按分钟分割时导出的日志文件如current-20221024T1428.log
//!     .file_split(PathBuf::from("./current.log"), Period::Day)
//!     // 如果既不配置输出文件 a)， 也不配置按指定间隔分割文件 b)，则默认输出到stderr
//!     // a) 和 b) 互斥，写在后面的生效，比如这里就是file_split生效
//!     .max_log_level(LevelFilter::Info)
//!     .build()
//!     .expect("logger build failed");
//! // 初始化
//! logger.init().expect("set logger failed");
//! ```
//!
//! ### ftlog与log
//!
//! ftlog与rust的log生态不兼容，建议删除掉原来的日志库。特别是**不要让两个日志库导出到同一个地方**，否则两个日志生态会同时打印，导致日志不可读。
//!
//! 删除原有的log库，比如：
//! ```toml
//! # log = "*"
//! # log4rs = "*"
//! # simple-logging = "*"
//! # env_logger = "*"
//! ```
//!
//! ## 用法
//!
//! ```rust
//! trace!("Hello world!");
//! debug!("Hello world!");
//! info!("Hello world!");
//! warn!("Hello world!");
//! error!("Hello world!");
//! ```
//!
//! 在main最后加入flush，否则在程序结束时未写入的日志会丢失：
//! ```rust
//! ftlog::logger().flush();
//! ```
//!
//! 参见 `examples/ftlog.rs`
//!
//! ### 带间隔的日志
//! 本库支持受限写入的功能。
//!
//! ```rust
//! info!(limit: 3000, "limit running{} !", i);
//! ```
//! 上面这一行日志会有最小3000毫秒的间隔，也就是最多3000ms一条。
//!
//! 不同代码行的日志间隔是互相独立的，互不影响。
//!
//! 后台线程以(模块、文件名、代码行)为key来记录上次日志打印时间，小于间隔则跳过。
//!
//! ### 日志分割
//! 本库支持日志按时间分割，自动适配本地时区，可选分割时间精度如下：
//!
//! - 分钟 `Period::Minute`
//! - 小时 `Period::Hour`
//! - 日 `Period::Day`
//! - 月 `Period::Month`
//! - 年 `Period::Year`
//!
//! ```rust
//! use ftlog::writer::file_split::Period;
//!
//! let logger = LogBuilder::new()
//!     .file_split(PathBuf::from("./current.log"), Period::Minute)
//!     .build()
//!     .unwrap();
//! logger.init().unwrap();
//! ```
//!
//! 设置按分钟分割时，输出日志文件格式为 `mylog-{MMMM}{YY}{DD}T{hh}{mm}.log`，其中`mylog`为调用时指定的文件名。如果未指定拓展名则在指定文件名后面加时间标签。
//!
//! 时间戳只保留到分割的时间精度，如按日分割保存的文件名格式为 `log-{MMMM}{YY}{DD}.log`。
//!
//! 输出的日志示例：
//! ```sh
//! $ ls
//! # 按分钟分割
//! current-20221026T1351.log
//! current-20221026T1352.log
//! current-20221026T1353.log
//! # 按小时分割
//! current-20221026T13.log
//! current-20221026T14.log
//! # 按日分割
//! current-20221026.log
//! current-20221027.log
//! # 按月分割
//! current-202210.log
//! current-202211.log
//! # 按年分割
//! current-2022.log
//! current-2023.log
//! # 设置的保存文件名无拓展时(如"./log")，则直接在文件名末尾加时间
//! log-20221026T1351
//! log-20221026T1352
//! log-20221026T1353
//! ```
//!
//!
//! ## 其他库日志不输出的问题
//! ftlog为了防止日志输出太多卡住服务器，特意开发了limit: 3000的功能，
//! 这个功能要改一个结构体导致跟官方的log库不兼容。
//!
//! 为了让用了官方`log::info!`的代码也可以输出日志（通常是引用的其他库的代码日志），需要在项目中配置logging，比如使用`simple-logging`, `env-logger`, `log4rs`等。
//!
//! 这里以`simple-logging`为例, `Cargo.toml` 加上
//! ```toml
//! log = "*"
//! simple-logging = "*"
//! ```
//!
//! main函数初始化加上这一行代码，即可以把官方标准log::info!的日志输出到stderr
//! ```rust
//! simple_logging::log_to_stderr(log::LevelFilter::Info);
//! ```
//!
//! 建议指定crate的版本，并配置ftlog和rust标准log库输出到不同地方（如打印到两个不同文件），否则两个库可能同时写入导致日志错乱。
use std::fs::OpenOptions;
use std::io::{stderr, Error as IoError, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{cmp, fmt, mem};

use fxhash::FxHashMap;
use time::{get_time, Timespec};

use self::writer::file_split::{FileSplit, Period};

#[macro_use]
pub mod macros;
pub mod writer;

#[macro_use]
extern crate cfg_if;

#[derive(Clone, Debug)]
enum LoggerInput {
    LogMsg((Timespec, String, u32, u64)),
    Flush,
    Quit,
}

#[derive(Clone, Debug)]
enum LoggerOutput {
    Flushed,
}

pub struct Logger {
    format: Box<dyn Fn(&Record) -> String + Sync + Send>,
    level: LevelFilter,
    queue: crossbeam_channel::Sender<LoggerInput>,
    notification: crossbeam_channel::Receiver<LoggerOutput>,
    worker_thread: Option<std::thread::JoinHandle<()>>,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

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
        self.level >= metadata.level()
    }

    fn log(&self, record: &Record) {
        let log_msg = (self.format)(record);
        let limit_key = if record.limit == 0 {
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
                hash_id = hash_id.unchecked_mul(16777619) ^ (record.line.unwrap_or(0) as u64);
            }
            hash_id
        };
        self.queue
            .send(LoggerInput::LogMsg((
                get_time(),
                log_msg,
                record.limit,
                limit_key,
            )))
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

pub struct LogBuilder {
    format: Box<dyn Fn(&Record) -> String + Sync + Send>,
    level: LevelFilter,
    writer: Option<Box<dyn Write + Send>>,
}

impl LogBuilder {
    #[inline]
    pub fn new() -> LogBuilder {
        LogBuilder {
            format: Box::new(|record: &Record| {
                format!(
                    "{} {}/{}:{} {}",
                    record.level(),
                    std::thread::current().name().unwrap_or_default(),
                    record.file().unwrap_or(""),
                    record.line().unwrap_or(0),
                    record.args()
                )
            }),
            level: LevelFilter::Info,
            writer: None,
        }
    }

    #[inline]
    pub fn format<F: 'static>(mut self, format: F) -> LogBuilder
    where
        F: Fn(&Record) -> String + Sync + Send,
    {
        self.format = Box::new(format);
        self
    }

    #[inline]
    pub fn file<T: Into<PathBuf>>(mut self, path: T) -> LogBuilder {
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.into());
        self.writer = Some(Box::new(f.unwrap()));
        self
    }

    #[inline]
    pub fn file_split<T: Into<PathBuf>>(mut self, path: T, period: Period) -> LogBuilder {
        self.writer = Some(Box::new(FileSplit::new(path.into(), period)));
        self
    }

    #[inline]
    pub fn max_log_level(mut self, level: LevelFilter) -> LogBuilder {
        self.level = level;
        self
    }

    pub fn build(self) -> Result<Logger, IoError> {
        let (sync_sender, receiver) = crossbeam_channel::unbounded();
        let (notification_sender, notification_receiver) = crossbeam_channel::bounded(1);
        let worker_thread = std::thread::Builder::new()
            .name("logger".to_string())
            .spawn(move || {
                let mut writer: Box<dyn Write> = match self.writer {
                    Some(w) => w,
                    _ => Box::new(stderr()) as Box<dyn Write>,
                };
                let mut last_log = FxHashMap::default();
                let mut missed_log = FxHashMap::default();
                loop {
                    match receiver.recv() {
                        Ok(LoggerInput::LogMsg((tm, msg, limit, limit_key))) => {
                            let now = get_time();
                            let now_nanos = now.sec * 1000_000_000 + now.nsec as i64;
                            if limit > 0 {
                                let missed_entry = missed_log.entry(limit_key).or_insert_with(||{0_i64});
                                if let Some(last_nanos) = last_log.get(&limit_key) {
                                    if now_nanos - last_nanos < limit as i64 * 1_000_000 {
                                        *missed_entry += 1;
                                        continue;
                                    }
                                }
                                last_log.insert(limit_key, now_nanos);
                                let delay = (now_nanos - (tm.sec * 1000_000_000 + tm.nsec as i64)) / 1_000_000;
                                let tm = time::at(tm);
                                let tm_millisec = tm.tm_nsec / 1_000_000;
                                writeln!(&mut writer, "{:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3}{:>+03} {}ms {} {}",
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
                                         msg).expect("logger write message failed");
                                *missed_entry = 0;
                            } else {
                                let delay = (now_nanos - (tm.sec * 1000_000_000 + tm.nsec as i64)) / 1_000_000;
                                let tm = time::at(tm);
                                let tm_millisec = tm.tm_nsec / 1_000_000;
                                writeln!(&mut writer, "{:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3}{:>+03} {}ms {}",
                                         tm.tm_year + 1900,
                                         tm.tm_mon + 1,
                                         tm.tm_mday,
                                         tm.tm_hour,
                                         tm.tm_min,
                                         tm.tm_sec,
                                         tm_millisec,
                                         tm.tm_utcoff / 3600,
                                         delay,
                                         msg).expect("logger write message failed");
                            }
                        }
                        Ok(LoggerInput::Flush) => {
                            notification_sender
                                .send(LoggerOutput::Flushed)
                                .expect("logger notification failed");
                        }
                        Ok(LoggerInput::Quit) => {
                            break;
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

impl Default for LogBuilder {
    #[inline]
    fn default() -> Self {
        LogBuilder::new()
    }
}

// Any platform without atomics is unlikely to have multiple cores, so
// writing via Cell will not be a race condition.
// The LOGGER static holds a pointer to the global logger. It is protected by
// the STATE static which determines whether LOGGER has been initialized yet.
static mut LOGGER: &dyn Log = &NopLogger;

static STATE: AtomicUsize = AtomicUsize::new(0);

// There are three different states that we care about: the logger's
// uninitialized, the logger's initializing (set_logger's been called but
// LOGGER hasn't actually been set yet), or the logger's active.
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

static LOG_LEVEL_NAMES: [&str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

static SET_LOGGER_ERROR: &str = "attempted to set a logger after the logging system \
                                 was already initialized";
static LEVEL_PARSE_ERROR: &str =
    "attempted to convert a string that doesn't match an existing log level";

/// An enum representing the available verbosity levels of the logger.
///
/// Typical usage includes: checking if a certain `Level` is enabled with
/// [`log_enabled!`](macro.log_enabled.html), specifying the `Level` of
/// [`log!`](macro.log.html), and comparing a `Level` directly to a
/// [`LevelFilter`](enum.LevelFilter.html).
#[repr(usize)]
#[derive(Copy, Eq, Debug, Hash)]
pub enum Level {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    // This way these line up with the discriminants for LevelFilter below
    // This works because Rust treats field-less enums the same way as C does:
    // https://doc.rust-lang.org/reference/items/enumerations.html#custom-discriminant-values-for-field-less-enumerations
    Error = 1,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl Clone for Level {
    #[inline]
    fn clone(&self) -> Level {
        *self
    }
}

impl PartialEq for Level {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd for Level {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &Level) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &Level) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &Level) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &Level) -> bool {
        *self as usize >= *other as usize
    }
}

impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }

    #[inline]
    fn lt(&self, other: &LevelFilter) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &LevelFilter) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &LevelFilter) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &LevelFilter) -> bool {
        *self as usize >= *other as usize
    }
}

impl Ord for Level {
    #[inline]
    fn cmp(&self, other: &Level) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

fn ok_or<T, E>(t: Option<T>, e: E) -> Result<T, E> {
    match t {
        Some(t) => Ok(t),
        None => Err(e),
    }
}

// Reimplemented here because std::ascii is not available in libcore
fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    fn to_ascii_uppercase(c: u8) -> u8 {
        if c >= b'a' && c <= b'z' {
            c - b'a' + b'A'
        } else {
            c
        }
    }

    if a.len() == b.len() {
        a.bytes()
            .zip(b.bytes())
            .all(|(a, b)| to_ascii_uppercase(a) == to_ascii_uppercase(b))
    } else {
        false
    }
}

impl FromStr for Level {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<Level, Self::Err> {
        ok_or(
            LOG_LEVEL_NAMES
                .iter()
                .position(|&name| eq_ignore_ascii_case(name, level))
                .into_iter()
                .filter(|&idx| idx != 0)
                .map(|idx| Level::from_usize(idx).unwrap())
                .next(),
            ParseLevelError(()),
        )
    }
}

impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self.as_str())
    }
}

impl Level {
    fn from_usize(u: usize) -> Option<Level> {
        match u {
            1 => Some(Level::Error),
            2 => Some(Level::Warn),
            3 => Some(Level::Info),
            4 => Some(Level::Debug),
            5 => Some(Level::Trace),
            _ => None,
        }
    }

    /// Returns the most verbose logging level.
    #[inline]
    pub fn max() -> Level {
        Level::Trace
    }

    /// Converts the `Level` to the equivalent `LevelFilter`.
    #[inline]
    pub fn to_level_filter(&self) -> LevelFilter {
        LevelFilter::from_usize(*self as usize).unwrap()
    }

    /// Returns the string representation of the `Level`.
    ///
    /// This returns the same string as the `fmt::Display` implementation.
    pub fn as_str(&self) -> &'static str {
        LOG_LEVEL_NAMES[*self as usize]
    }

    /// Iterate through all supported logging levels.
    ///
    /// The order of iteration is from more severe to less severe log messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use log::Level;
    ///
    /// let mut levels = Level::iter();
    ///
    /// assert_eq!(Some(Level::Error), levels.next());
    /// assert_eq!(Some(Level::Trace), levels.last());
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        (1..6).map(|i| Self::from_usize(i).unwrap())
    }
}

/// An enum representing the available verbosity level filters of the logger.
///
/// A `LevelFilter` may be compared directly to a [`Level`]. Use this type
/// to get and set the maximum log level with [`max_level()`] and [`set_max_level`].
///
/// [`Level`]: enum.Level.html
/// [`max_level()`]: fn.max_level.html
/// [`set_max_level`]: fn.set_max_level.html
#[repr(usize)]
#[derive(Copy, Eq, Debug, Hash)]
pub enum LevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

// Deriving generates terrible impls of these traits

impl Clone for LevelFilter {
    #[inline]
    fn clone(&self) -> LevelFilter {
        *self
    }
}

impl PartialEq for LevelFilter {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        other.eq(self)
    }
}

impl PartialOrd for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &LevelFilter) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &LevelFilter) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &LevelFilter) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &LevelFilter) -> bool {
        *self as usize >= *other as usize
    }
}

impl PartialOrd<Level> for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }

    #[inline]
    fn lt(&self, other: &Level) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &Level) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &Level) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &Level) -> bool {
        *self as usize >= *other as usize
    }
}

impl Ord for LevelFilter {
    #[inline]
    fn cmp(&self, other: &LevelFilter) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

impl FromStr for LevelFilter {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<LevelFilter, Self::Err> {
        ok_or(
            LOG_LEVEL_NAMES
                .iter()
                .position(|&name| eq_ignore_ascii_case(name, level))
                .map(|p| LevelFilter::from_usize(p).unwrap()),
            ParseLevelError(()),
        )
    }
}

impl fmt::Display for LevelFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self.as_str())
    }
}

impl LevelFilter {
    fn from_usize(u: usize) -> Option<LevelFilter> {
        match u {
            0 => Some(LevelFilter::Off),
            1 => Some(LevelFilter::Error),
            2 => Some(LevelFilter::Warn),
            3 => Some(LevelFilter::Info),
            4 => Some(LevelFilter::Debug),
            5 => Some(LevelFilter::Trace),
            _ => None,
        }
    }

    /// Returns the most verbose logging level filter.
    #[inline]
    pub fn max() -> LevelFilter {
        LevelFilter::Trace
    }

    /// Converts `self` to the equivalent `Level`.
    ///
    /// Returns `None` if `self` is `LevelFilter::Off`.
    #[inline]
    pub fn to_level(&self) -> Option<Level> {
        Level::from_usize(*self as usize)
    }

    /// Returns the string representation of the `LevelFilter`.
    ///
    /// This returns the same string as the `fmt::Display` implementation.
    pub fn as_str(&self) -> &'static str {
        LOG_LEVEL_NAMES[*self as usize]
    }

    /// Iterate through all supported filtering levels.
    ///
    /// The order of iteration is from less to more verbose filtering.
    ///
    /// # Examples
    ///
    /// ```
    /// use log::LevelFilter;
    ///
    /// let mut levels = LevelFilter::iter();
    ///
    /// assert_eq!(Some(LevelFilter::Off), levels.next());
    /// assert_eq!(Some(LevelFilter::Trace), levels.last());
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..6).map(|i| Self::from_usize(i).unwrap())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum MaybeStaticStr<'a> {
    Static(&'static str),
    Borrowed(&'a str),
}

impl<'a> MaybeStaticStr<'a> {
    #[inline]
    fn get(&self) -> &'a str {
        match *self {
            MaybeStaticStr::Static(s) => s,
            MaybeStaticStr::Borrowed(s) => s,
        }
    }
}

/// The "payload" of a log message.
///
/// # Use
///
/// `Record` structures are passed as parameters to the [`log`][method.log]
/// method of the [`Log`] trait. Logger implementors manipulate these
/// structures in order to display log messages. `Record`s are automatically
/// created by the [`log!`] macro and so are not seen by log users.
///
/// Note that the [`level()`] and [`target()`] accessors are equivalent to
/// `self.metadata().level()` and `self.metadata().target()` respectively.
/// These methods are provided as a convenience for users of this structure.
///
/// # Example
///
/// The following example shows a simple logger that displays the level,
/// module path, and message of any `Record` that is passed to it.
///
/// ```edition2018
/// struct SimpleLogger;
///
/// impl log::Log for SimpleLogger {
///    fn enabled(&self, metadata: &log::Metadata) -> bool {
///        true
///    }
///
///    fn log(&self, record: &log::Record) {
///        if !self.enabled(record.metadata()) {
///            return;
///        }
///
///        println!("{}:{} -- {}",
///                 record.level(),
///                 record.target(),
///                 record.args());
///    }
///    fn flush(&self) {}
/// }
/// ```
///
/// [method.log]: trait.Log.html#tymethod.log
/// [`Log`]: trait.Log.html
/// [`log!`]: macro.log.html
/// [`level()`]: struct.Record.html#method.level
/// [`target()`]: struct.Record.html#method.target
#[derive(Clone, Debug)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    args: fmt::Arguments<'a>,
    module_path: Option<MaybeStaticStr<'a>>,
    file: Option<MaybeStaticStr<'a>>,
    line: Option<u32>,
    limit: u32,
}

// This wrapper type is only needed so we can
// `#[derive(Debug)]` on `Record`. It also
// provides a useful `Debug` implementation for
// the underlying `Source`.
#[cfg(feature = "kv_unstable")]
#[derive(Clone)]
struct KeyValues<'a>(&'a dyn kv::Source);

#[cfg(feature = "kv_unstable")]
impl<'a> fmt::Debug for KeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut visitor = f.debug_map();
        self.0.visit(&mut visitor).map_err(|_| fmt::Error)?;
        visitor.finish()
    }
}

impl<'a> Record<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }

    /// The message body.
    #[inline]
    pub fn args(&self) -> &fmt::Arguments<'a> {
        &self.args
    }

    /// Metadata about the log directive.
    #[inline]
    pub fn metadata(&self) -> &Metadata<'a> {
        &self.metadata
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.metadata.level()
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.metadata.target()
    }

    /// The module path of the message.
    #[inline]
    pub fn module_path(&self) -> Option<&'a str> {
        self.module_path.map(|s| s.get())
    }

    /// The module path of the message, if it is a `'static` string.
    #[inline]
    pub fn module_path_static(&self) -> Option<&'static str> {
        match self.module_path {
            Some(MaybeStaticStr::Static(s)) => Some(s),
            _ => None,
        }
    }

    /// The source file containing the message.
    #[inline]
    pub fn file(&self) -> Option<&'a str> {
        self.file.map(|s| s.get())
    }

    /// The module path of the message, if it is a `'static` string.
    #[inline]
    pub fn file_static(&self) -> Option<&'static str> {
        match self.file {
            Some(MaybeStaticStr::Static(s)) => Some(s),
            _ => None,
        }
    }

    /// The line containing the message.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// The line containing the message.
    #[inline]
    pub fn limit(&self) -> u32 {
        self.limit
    }

    /// The structured key-value pairs associated with the message.
    #[cfg(feature = "kv_unstable")]
    #[inline]
    pub fn key_values(&self) -> &dyn kv::Source {
        self.key_values.0
    }

    /// Create a new [`RecordBuilder`](struct.RecordBuilder.html) based on this record.
    #[cfg(feature = "kv_unstable")]
    #[inline]
    pub fn to_builder(&self) -> RecordBuilder {
        RecordBuilder {
            record: Record {
                metadata: Metadata {
                    level: self.metadata.level,
                    target: self.metadata.target,
                },
                args: self.args,
                module_path: self.module_path,
                file: self.file,
                line: self.line,
                key_values: self.key_values.clone(),
            },
        }
    }
}

/// Builder for [`Record`](struct.Record.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `RecordBuilder` can set the different parameters of `Record` object, and returns
/// the created object when `build` is called.
///
/// # Examples
///
/// ```edition2018
/// use log::{Level, Record};
///
/// let record = Record::builder()
///                 .args(format_args!("Error!"))
///                 .level(Level::Error)
///                 .target("myApp")
///                 .file(Some("server.rs"))
///                 .line(Some(144))
///                 .module_path(Some("server"))
///                 .build();
/// ```
///
/// Alternatively, use [`MetadataBuilder`](struct.MetadataBuilder.html):
///
/// ```edition2018
/// use log::{Record, Level, MetadataBuilder};
///
/// let error_metadata = MetadataBuilder::new()
///                         .target("myApp")
///                         .level(Level::Error)
///                         .build();
///
/// let record = Record::builder()
///                 .metadata(error_metadata)
///                 .args(format_args!("Error!"))
///                 .line(Some(433))
///                 .file(Some("app.rs"))
///                 .module_path(Some("server"))
///                 .build();
/// ```
#[derive(Debug)]
pub struct RecordBuilder<'a> {
    record: Record<'a>,
}

impl<'a> RecordBuilder<'a> {
    /// Construct new `RecordBuilder`.
    ///
    /// The default options are:
    ///
    /// - `args`: [`format_args!("")`]
    /// - `metadata`: [`Metadata::builder().build()`]
    /// - `module_path`: `None`
    /// - `file`: `None`
    /// - `line`: `None`
    ///
    /// [`format_args!("")`]: https://doc.rust-lang.org/std/macro.format_args.html
    /// [`Metadata::builder().build()`]: struct.MetadataBuilder.html#method.build
    #[inline]
    pub fn new() -> RecordBuilder<'a> {
        RecordBuilder {
            record: Record {
                args: format_args!(""),
                metadata: Metadata::builder().build(),
                module_path: None,
                file: None,
                line: None,
                limit: 0,
                #[cfg(feature = "kv_unstable")]
                key_values: KeyValues(&Option::None::<(kv::Key, kv::Value)>),
            },
        }
    }

    /// Set [`args`](struct.Record.html#method.args).
    #[inline]
    pub fn args(&mut self, args: fmt::Arguments<'a>) -> &mut RecordBuilder<'a> {
        self.record.args = args;
        self
    }

    /// Set [`metadata`](struct.Record.html#method.metadata). Construct a `Metadata` object with [`MetadataBuilder`](struct.MetadataBuilder.html).
    #[inline]
    pub fn metadata(&mut self, metadata: Metadata<'a>) -> &mut RecordBuilder<'a> {
        self.record.metadata = metadata;
        self
    }

    /// Set [`Metadata::level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut RecordBuilder<'a> {
        self.record.metadata.level = level;
        self
    }

    /// Set [`Metadata::target`](struct.Metadata.html#method.target)
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut RecordBuilder<'a> {
        self.record.metadata.target = target;
        self
    }

    /// Set [`module_path`](struct.Record.html#method.module_path)
    #[inline]
    pub fn module_path(&mut self, path: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.record.module_path = path.map(MaybeStaticStr::Borrowed);
        self
    }

    /// Set [`module_path`](struct.Record.html#method.module_path) to a `'static` string
    #[inline]
    pub fn module_path_static(&mut self, path: Option<&'static str>) -> &mut RecordBuilder<'a> {
        self.record.module_path = path.map(MaybeStaticStr::Static);
        self
    }

    /// Set [`file`](struct.Record.html#method.file)
    #[inline]
    pub fn file(&mut self, file: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.record.file = file.map(MaybeStaticStr::Borrowed);
        self
    }

    /// Set [`file`](struct.Record.html#method.file) to a `'static` string.
    #[inline]
    pub fn file_static(&mut self, file: Option<&'static str>) -> &mut RecordBuilder<'a> {
        self.record.file = file.map(MaybeStaticStr::Static);
        self
    }

    /// Set [`line`](struct.Record.html#method.line)
    #[inline]
    pub fn line(&mut self, line: Option<u32>) -> &mut RecordBuilder<'a> {
        self.record.line = line;
        self
    }
    #[inline]
    pub fn limit(&mut self, limit: u32) -> &mut RecordBuilder<'a> {
        self.record.limit = limit;
        self
    }

    /// Invoke the builder and return a `Record`
    #[inline]
    pub fn build(&self) -> Record<'a> {
        self.record.clone()
    }
}

/// Metadata about a log message.
///
/// # Use
///
/// `Metadata` structs are created when users of the library use
/// logging macros.
///
/// They are consumed by implementations of the `Log` trait in the
/// `enabled` method.
///
/// `Record`s use `Metadata` to determine the log message's severity
/// and target.
///
/// Users should use the `log_enabled!` macro in their code to avoid
/// constructing expensive log messages.
///
/// # Examples
///
/// ```edition2018
/// use log::{Record, Level, Metadata};
///
/// struct MyLogger;
///
/// impl log::Log for MyLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("{} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// # fn main(){}
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}

impl<'a> Metadata<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }
}

/// Builder for [`Metadata`](struct.Metadata.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `MetadataBuilder` can set the different parameters of a `Metadata` object, and returns
/// the created object when `build` is called.
///
/// # Example
///
/// ```edition2018
/// let target = "myApp";
/// use log::{Level, MetadataBuilder};
/// let metadata = MetadataBuilder::new()
///                     .level(Level::Debug)
///                     .target(target)
///                     .build();
/// ```
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MetadataBuilder<'a> {
    metadata: Metadata<'a>,
}

impl<'a> MetadataBuilder<'a> {
    /// Construct a new `MetadataBuilder`.
    ///
    /// The default options are:
    ///
    /// - `level`: `Level::Info`
    /// - `target`: `""`
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder {
            metadata: Metadata {
                level: Level::Info,
                target: "",
            },
        }
    }

    /// Setter for [`level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, arg: Level) -> &mut MetadataBuilder<'a> {
        self.metadata.level = arg;
        self
    }

    /// Setter for [`target`](struct.Metadata.html#method.target).
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut MetadataBuilder<'a> {
        self.metadata.target = target;
        self
    }

    /// Returns a `Metadata` object.
    #[inline]
    pub fn build(&self) -> Metadata<'a> {
        self.metadata.clone()
    }
}

/// A trait encapsulating the operations required of a logger.
pub trait Log: Sync + Send {
    /// Determines if a log message with the specified metadata would be
    /// logged.
    ///
    /// This is used by the `log_enabled!` macro to allow callers to avoid
    /// expensive computation of log message arguments if the message would be
    /// discarded anyway.
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Logs the `Record`.
    ///
    /// Note that `enabled` is *not* necessarily called before this method.
    /// Implementations of `log` should perform all necessary filtering
    /// internally.
    fn log(&self, record: &Record);

    /// Flushes any buffered records.
    fn flush(&self);
}

// Just used as a dummy initial value for LOGGER
struct NopLogger;

impl Log for NopLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        false
    }

    fn log(&self, _: &Record) {}
    fn flush(&self) {}
}

impl<T> Log for &'_ T
where
    T: ?Sized + Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        (**self).enabled(metadata)
    }

    fn log(&self, record: &Record) {
        (**self).log(record)
    }
    fn flush(&self) {
        (**self).flush()
    }
}

#[cfg(feature = "std")]
impl<T> Log for std::boxed::Box<T>
where
    T: ?Sized + Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.as_ref().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.as_ref().log(record)
    }
    fn flush(&self) {
        self.as_ref().flush()
    }
}

#[cfg(feature = "std")]
impl<T> Log for std::sync::Arc<T>
where
    T: ?Sized + Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.as_ref().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.as_ref().log(record)
    }
    fn flush(&self) {
        self.as_ref().flush()
    }
}

/// Sets the global maximum log level.
///
/// Generally, this should only be called by the active logging implementation.
///
/// Note that `Trace` is the maximum level, because it provides the maximum amount of detail in the emitted logs.
#[inline]
pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::Relaxed)
}

/// Returns the current maximum log level.
///
/// The [`log!`], [`error!`], [`warn!`], [`info!`], [`debug!`], and [`trace!`] macros check
/// this value and discard any message logged at a higher level. The maximum
/// log level is set by the [`set_max_level`] function.
///
/// [`log!`]: macro.log.html
/// [`error!`]: macro.error.html
/// [`warn!`]: macro.warn.html
/// [`info!`]: macro.info.html
/// [`debug!`]: macro.debug.html
/// [`trace!`]: macro.trace.html
/// [`set_max_level`]: fn.set_max_level.html
#[inline(always)]
pub fn max_level() -> LevelFilter {
    // Since `LevelFilter` is `repr(usize)`,
    // this transmute is sound if and only if `MAX_LOG_LEVEL_FILTER`
    // is set to a usize that is a valid discriminant for `LevelFilter`.
    // Since `MAX_LOG_LEVEL_FILTER` is private, the only time it's set
    // is by `set_max_level` above, i.e. by casting a `LevelFilter` to `usize`.
    // So any usize stored in `MAX_LOG_LEVEL_FILTER` is a valid discriminant.
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}

/// Sets the global logger to a `Box<Log>`.
///
/// This is a simple convenience wrapper over `set_logger`, which takes a
/// `Box<Log>` rather than a `&'static Log`. See the documentation for
/// [`set_logger`] for more details.
///
/// Requires the `std` feature.
///
/// # Errors
///
/// An error is returned if a logger has already been set.
///
/// [`set_logger`]: fn.set_logger.html
// #[cfg(all(feature = "std", atomic_cas))]
pub fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), SetLoggerError> {
    set_logger_inner(|| Box::leak(logger))
}

/// Sets the global logger to a `&'static Log`.
///
/// This function may only be called once in the lifetime of a program. Any log
/// events that occur before the call to `set_logger` completes will be ignored.
///
/// This function does not typically need to be called manually. Logger
/// implementations should provide an initialization method that installs the
/// logger internally.
///
/// # Availability
///
/// This method is available even when the `std` feature is disabled. However,
/// it is currently unavailable on `thumbv6` targets, which lack support for
/// some atomic operations which are used by this function. Even on those
/// targets, [`set_logger_racy`] will be available.
///
/// # Errors
///
/// An error is returned if a logger has already been set.
///
/// # Examples
///
/// ```edition2018
/// use log::{error, info, warn, Record, Level, Metadata, LevelFilter};
///
/// static MY_LOGGER: MyLogger = MyLogger;
///
/// struct MyLogger;
///
/// impl log::Log for MyLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("{} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// # fn main(){
/// log::set_logger(&MY_LOGGER).unwrap();
/// log::set_max_level(LevelFilter::Info);
///
/// info!("hello log");
/// warn!("warning");
/// error!("oops");
/// # }
/// ```
///
/// [`set_logger_racy`]: fn.set_logger_racy.html
// #[cfg(atomic_cas)]
pub fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError> {
    set_logger_inner(|| logger)
}

// #[cfg(atomic_cas)]
fn set_logger_inner<F>(make_logger: F) -> Result<(), SetLoggerError>
where
    F: FnOnce() -> &'static dyn Log,
{
    let old_state = match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            unsafe {
                LOGGER = make_logger();
            }
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            while STATE.load(Ordering::SeqCst) == INITIALIZING {
                // TODO: replace with `hint::spin_loop` once MSRV is 1.49.0.
                #[allow(deprecated)]
                std::sync::atomic::spin_loop_hint();
            }
            Err(SetLoggerError(()))
        }
        _ => Err(SetLoggerError(())),
    }
}

/// A thread-unsafe version of [`set_logger`].
///
/// This function is available on all platforms, even those that do not have
/// support for atomics that is needed by [`set_logger`].
///
/// In almost all cases, [`set_logger`] should be preferred.
///
/// # Safety
///
/// This function is only safe to call when no other logger initialization
/// function is called while this function still executes.
///
/// This can be upheld by (for example) making sure that **there are no other
/// threads**, and (on embedded) that **interrupts are disabled**.
///
/// It is safe to use other logging functions while this function runs
/// (including all logging macros).
///
/// [`set_logger`]: fn.set_logger.html
pub unsafe fn set_logger_racy(logger: &'static dyn Log) -> Result<(), SetLoggerError> {
    match STATE.load(Ordering::SeqCst) {
        UNINITIALIZED => {
            LOGGER = logger;
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            // This is just plain UB, since we were racing another initialization function
            unreachable!("set_logger_racy must not be used with other initialization functions")
        }
        _ => Err(SetLoggerError(())),
    }
}

/// The type returned by [`set_logger`] if [`set_logger`] has already been called.
///
/// [`set_logger`]: fn.set_logger.html
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct SetLoggerError(());

impl fmt::Display for SetLoggerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(SET_LOGGER_ERROR)
    }
}

// The Error trait is not available in libcore
#[cfg(feature = "std")]
impl error::Error for SetLoggerError {}

/// The type returned by [`from_str`] when the string doesn't match any of the log levels.
///
/// [`from_str`]: https://doc.rust-lang.org/std/str/trait.FromStr.html#tymethod.from_str
#[allow(missing_copy_implementations)]
#[derive(Debug, PartialEq)]
pub struct ParseLevelError(());

impl fmt::Display for ParseLevelError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(LEVEL_PARSE_ERROR)
    }
}

// The Error trait is not available in libcore
#[cfg(feature = "std")]
impl error::Error for ParseLevelError {}

/// Returns a reference to the logger.
///
/// If a logger has not been set, a no-op implementation is returned.
pub fn logger() -> &'static dyn Log {
    if STATE.load(Ordering::SeqCst) != INITIALIZED {
        static NOP: NopLogger = NopLogger;
        &NOP
    } else {
        unsafe { LOGGER }
    }
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
#[cfg(not(feature = "kv_unstable"))]
pub fn __private_api_log(
    args: fmt::Arguments,
    level: Level,
    &(target, module_path, file, line, limit): &(&str, &'static str, &'static str, u32, u32),
    kvs: Option<&[(&str, &str)]>,
) {
    if kvs.is_some() {
        panic!(
            "key-value support is experimental and must be enabled using the `kv_unstable` feature"
        )
    }

    logger().log(
        &Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .module_path_static(Some(module_path))
            .file_static(Some(file))
            .line(Some(line))
            .limit(limit)
            .build(),
    );
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub fn __private_api_enabled(level: Level, target: &str) -> bool {
    logger().enabled(&Metadata::builder().level(level).target(target).build())
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub mod __private_api {
    pub use std::option::Option;
}

/// The statically resolved maximum log level.
///
/// See the crate level documentation for information on how to configure this.
///
/// This value is checked by the log macros, but not by the `Log`ger returned by
/// the [`logger`] function. Code that manually calls functions on that value
/// should compare the level against this value.
///
/// [`logger`]: fn.logger.html
pub const STATIC_MAX_LEVEL: LevelFilter = MAX_LEVEL_INNER;

cfg_if! {
    if #[cfg(all(not(debug_assertions), feature = "release_max_level_off"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Off;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_error"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Error;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_warn"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Warn;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_info"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Info;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_debug"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Debug;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_trace"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Trace;
    } else if #[cfg(feature = "max_level_off")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Off;
    } else if #[cfg(feature = "max_level_error")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Error;
    } else if #[cfg(feature = "max_level_warn")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Warn;
    } else if #[cfg(feature = "max_level_info")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Info;
    } else if #[cfg(feature = "max_level_debug")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Debug;
    } else {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Trace;
    }
}
