#![feature(unchecked_math)]
//! [![Build Status](https://github.com/nonconvextech/ftlog/workflows/CI%20%28Linux%29/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
//! ![License](https://img.shields.io/crates/l/ftlog.svg)
//! [![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
//! [![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)
//!
//! 普通的日志库受到磁盘io和系统pipe影响，单线程顺序写入单条速度大概要2500ns（SSD），如果碰到io抖动或者慢磁盘，日志会是低延时交易的主要瓶颈。
//! 本库先把日志send到channel，再启动后台单独线程recv并且磁盘写入，测试速度在300ns左右。
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
//! ftlog = "0.2.0"
//! log = "0.4.17"
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
//! use ftlog::{LogBuilder, writer::file_split::Period, LevelFilter};
//!
//! // 完整用法
//! // 配置logger
//! let logger = LogBuilder::new()
//!     //这里可以定义自己的格式，时间格式暂时不可以自定义
//!     // .format(format)
//!     // a) 这里可以配置输出到文件
//!     .file(std::path::PathBuf::from("./current.log"))
//!     // b) 这里可以配置输出到文件，并且按指定间隔分割。这里导出的按天分割日志文件如current-20221024.log
//!     // 配置为按分钟分割时导出的日志文件如current-20221024T1428.log
//!     .file_split(std::path::PathBuf::from("./current.log"), Period::Day)
//!     // 如果既不配置输出文件 a)， 也不配置按指定间隔分割文件 b)，则默认输出到stderr
//!     // a) 和 b) 互斥，写在后面的生效，比如这里就是file_split生效
//!     .max_log_level(LevelFilter::Info)
//!     .build()
//!     .expect("logger build failed");
//! // 初始化
//! logger.init().expect("set logger failed");
//! ```
//!
//! ## 用法
//!
//! ```rust, ignore
//! trace!("Hello world!");
//! debug!("Hello world!");
//! info!("Hello world!");
//! warn!("Hello world!");
//! error!("Hello world!");
//! ```
//!
//! 在main最后加入flush，否则在程序结束时未写入的日志会丢失：
//! ```rust, ignore
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
//! use std::path::PathBuf;
//! use ftlog::{LogBuilder, writer::file_split::Period};
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

pub use log::{debug, error, info, log, log_enabled, trace, warn, LevelFilter, Record};

use std::fs::OpenOptions;
use std::io::{stderr, Error as IoError, Write};
use std::path::PathBuf;

use fxhash::FxHashMap;
use log::kv::Key;
use log::{set_boxed_logger, set_max_level, Log, Metadata, SetLoggerError};
use time::{get_time, Timespec};

use self::writer::file_split::{FileSplit, Period};

pub mod writer;

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
        let limit = record
            .key_values()
            .get(Key::from_str("limit"))
            .map(|x| x.to_u64())
            .flatten()
            .unwrap_or(0) as u32;
        let log_msg = (self.format)(record);
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
            .send(LoggerInput::LogMsg((get_time(), log_msg, limit, limit_key)))
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
