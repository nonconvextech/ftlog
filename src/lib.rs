#![feature(unchecked_math)]
//! [![Build Status](https://github.com/nonconvextech/ftlog/workflows/CI%20%28Linux%29/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
//! ![License](https://img.shields.io/crates/l/ftlog.svg)
//! [![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
//! [![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)
//!
//! 普通的日志库受到磁盘io和系统pipe影响，单线程顺序写入单条速度大概要2500ns（SSD），如果碰到io抖动或者慢磁盘，日志会是低延时交易的主要瓶颈。
//! 本库先把日志send到channel，再启动后台单独线程recv并且磁盘写入，测试速度在300ns左右。
//!
//! **CAUTION**: this crate use `unchecked_math` unstable feature and `unsafe` code. Only use this crate in rust `nightly` channel.
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
//! ftlog = "0.2.0"
//! ```
//!
//! 在main开头加入配置：
//! ```rust
//! // 最简配置用法，使用默认配置，输出到stderr
//! ftlog::builder().build().unwrap().init().unwrap();
//!
//! ```
//!
//! 默认的配置会把日志输出到stderr，程序启动的时候要执行bin/yourjob &>> logs/server.log来把日志定向到文件。（注意shell要用/bin/bash不要用/bin/sh。/bin/sh在ubuntu下有问题，不标准)
//!
//! 也可以直接输出到固定文件，完整的配置用法如下:
//!
//! ```rust,no_run
//! use ftlog::{appender::{Period, FileAppender}, LevelFilter, Record, FtLogFormat};
//!
//! // 完整用法
//!
//! // Custom format
//!
//! // Here is the dumbest implementation that format to String in worker thread
//! // A better way is to store required field and send to log thread to format into string
//! struct StringFormatter;
//! impl FtLogFormat for StringFormatter {
//!     fn msg(&self, record: &Record) -> Box<dyn Send + Sync + std::fmt::Display> {
//!         Box::new(format!(
//!             "{} {}/{}:{} {}",
//!             record.level(),
//!             std::thread::current().name().unwrap_or_default(),
//!             record.file().unwrap_or(""),
//!             record.line().unwrap_or(0),
//!             record.args()
//!         ))
//!     }
//! }
//!
//! // 配置logger
//! let logger = ftlog::builder()
//!     // 这里可以定义自己的格式，时间格式暂时不可以自定义
//!     // 只能设置全局格式，不允许对不同输出目标单独设定格式
//!     .format(StringFormatter)
//!     .root(FileAppender::rotate("./current.log", Period::Day))
//!     // 将ftlog::appender下的日志输出到默认日志输出，并过滤Warn以下级别
//!     .filter("ftlog::appender", None, LevelFilter::Warn)
//!     // 将ftlog::appender::file下的日志输出到后面定义的"ftlog-appender"输出方式，默认过滤级别
//!     .filter("ftlog::appender::file", "ftlog-appender", None)
//!     // 定义新的日志输出方式，写出数据结构只要满足`Send + Write + 'static`
//!     .appender("ftlog-appender", FileAppender::rotate("ftlog.log", Period::Day))
//!     // 全局日志过滤等级
//!     .max_log_level(LevelFilter::Info)
//!     .build()
//!     .expect("logger build failed");
//! // 初始化
//! logger.init().expect("set logger failed");
//! ```
//!
//! ## 用法
//!
//! ```rust,no_run
//! use ftlog::{trace, debug};
//! use log::{info, warn ,error};
//! trace!("Hello world!");
//! debug!("Hello world!");
//! info!("Hello world!");
//! warn!("Hello world!");
//! error!("Hello world!");
//! ```
//!
//! 在main最后加入flush，否则在程序结束时未写入的日志会丢失：
//! ```rust,no_run
//! ftlog::logger().flush();
//! ```
//!
//! 参见 `examples/ftlog.rs`
//!
//! ### 带间隔的日志
//! 本库支持受限写入的功能。
//!
//! ```rust
//! # use ftlog::info;
//! info!(limit=3000; "limit running{} !", 1);
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
//! use ftlog::{appender::{Period, FileAppender}};
//!
//! let logger = ftlog::builder()
//!     .root(FileAppender::rotate("./current.log", Period::Minute))
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
//! #### 自动清理分割后的日志
//!
//! 在设置了日志按时间分割的基础上，可以设置自动清理日志。按照上次修改时间清理ftlog产生的日志文件。
//! 注意：自动清理使用文件名匹配，文件名与日志文件名格式完全一致的文件也会被清理。
//!
//! ```rust
//! use ftlog::{appender::{Period, FileAppender, Duration}};
//!
//! // 这个例子中，文件名满足current-\d{8}T\d{4}.log的文件将被清理
//! let appender = FileAppender::rotate_with_expire("./current.log", Period::Minute, Duration::seconds(180));
//! let logger = ftlog::builder()
//!     .root(appender)
//!     .build()
//!     .unwrap();
//! logger.init().unwrap();
//! ```
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
                            std::thread::sleep(Duration::from_micros(200));
                        }
                        Err(e) => {
                            panic!(
                                "sender closed without sending a Quit first, this is a bug, {}",
                                e
                            );
                        }
                    }
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
