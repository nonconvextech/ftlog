# ftlog

[![Build Status](https://github.com/nonconvextech/ftlog/workflows/ftlog/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
![License](https://img.shields.io/crates/l/ftlog.svg)
[![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
[![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)

普通的日志库受到磁盘io和系统pipe影响，单线程顺序写入单条速度大概要2500ns（SSD），如果碰到io抖动或者慢磁盘，日志会是低延时交易的主要瓶颈。
本库先把日志send到channel，再启动后台单独线程recv并且磁盘写入，测试速度在300ns左右。

## 用法

在 `Cargo.toml` 中加入引用
```toml
ftlog = "0.2"
```

在`main`函数开头加入配置：
```rust
// ftlog 导出了log库常用的宏
use ftlog::appender::FileAppender;
use ftlog::{debug, trace};
use log::{error, info, warn};

// 最简配置，使用默认设置
// root函数传入None则打印日志到stderr
let dest = FileAppender::new("./current.log");
ftlog::builder().root(dest).build().unwrap().init().unwrap();

trace!("Hello world!");
debug!("Hello world!");
info!("Hello world!");
warn!("Hello world!");
error!("Hello world!");

```

以下是一个功能更丰富的例子：

```rust
use ftlog::{
    appender::{Duration, FileAppender, Period},
    FtLogFormat, LevelFilter, Record,
};

// 配置logger
let logger = ftlog::builder()
    // 全局日志过滤等级
    .max_log_level(LevelFilter::Info)
    // 设置全局日志格式,为了性能时间戳格式不允许修改
    .format(FtLogFormatter)
    // 设置日志消息处理队列长度上限，避免大量未处理日志导致高内存占用
    // 设成 `false` 会在队列满时丢弃新日志
    // 设成 `true` 会在队列满时阻塞线程，等待日志线程
    // 以下是默认设置
    // 也可以用 unbound() 设置成不定长消息队列，消息特别多时会按需申请内存
    .bounded(100_000, false) // .unbounded()
    // 配置默认输出位置，传入None则输出到stderr
    .root(FileAppender::rotate_with_expire(
        "./current.log",
        Period::Day,
        Duration::days(7),
    ))
    // 将 ftlog::appender 下的日志输出到后面定义的"ftlog-appender"，即"ftlog-appender.log"文件
    .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
    .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
    .build()
    .expect("logger build failed");
// 将ftlog设置为全局logger
logger.init().expect("set logger failed");
```

更多例子请参见 `./examples`。

### 日志格式

```plain
2022-04-08 19:20:48.190+08 298ms <<这里是日志的延时，正常这个数为0，如果太大说明channel堵塞了 INFO main [src/ftlog.rs:14] 14575
```

其中 `298ms` 表示调用日志打印到日志专属线程开始处理日志消息之间的延迟，正常应为0ms。

```plain
2022-04-10 21:27:15.996+08 0ms 2 <<这表示丢弃了2条日志，说明使用了limit语法 INFO main [src/main.rs:29] limit running3 !
```

### 带间隔的日志

本库支持限制单条日志打印的频率。

不同代码行的日志间隔是互相独立的，互不影响。

后台线程以(模块、文件名、代码行)为key来记录上次日志打印时间，小于间隔则跳过。

下面这一行日志会有最小3000毫秒的间隔，也就是最多3000ms一条。
```rust
info!(limit=3000; "limit running{} !", 1);
```

```markdown
2022-04-10 21:27:10.996+08 0ms 0 INFO main [src/main.rs:29] limit running 3s !
2022-04-10 21:27:15.996+08 0ms 2 INFO main [src/main.rs:29] limit running 3s !
```
上面的数字2表示，在两条日志中间，还有2条被丢弃的日志。

### 日志分割

本库支持日志按时间分割，自动适配本地时区，可选分割时间精度如下：

- 分钟 `Period::Minute`
- 小时 `Period::Hour`
- 日 `Period::Day`
- 月 `Period::Month`
- 年 `Period::Year`

日志分割通过`FileAppender`设置，使用示例如下：

```rust
use ftlog::appender::{FileAppender, Period};

let logger = ftlog::builder()
    .root(FileAppender::rotate("./mylog.log", Period::Minute))
    .build()
    .unwrap();
logger.init().unwrap();
```

设置按分钟分割时，输出日志文件格式为 `mylog-{MMMM}{YY}{DD}T{hh}{mm}.log`，其中`mylog`为调用时指定的文件名。如果未指定拓展名则在指定文件名后面加时间标签。

时间戳只保留到分割的时间精度，如按日分割保存的文件名格式为 `log-{MMMM}{YY}{DD}.log`。

输出的日志示例：
```sh
$ ls
# 按分钟分割
current-20221026T1351.log
# 按小时分割
current-20221026T13.log
# 按日分割
current-20221026.log
# 按月分割
current-202210.log
# 按年分割
current-2022.log
# 设置的保存文件名无拓展时(如"./log")，则直接在文件名末尾加时间
log-20221026T1351
```

#### 自动清理分割后的日志

在设置了日志按时间分割的基础上，可以设置自动清理日志。按照上次修改时间清理ftlog产生的日志文件。

**注意**：自动清理使用文件名匹配，文件名与日志文件名格式完全一致的文件也会被清理。

```rust
use ftlog::{appender::{Period, FileAppender, Duration}};

// 这个例子中，文件名满足current-\d{8}T\d{4}.log的文件将被清理
// `another-\d{8}T\d{4}.log`, `current-\d{8}T\d{4}` 等等由于文件名不匹配不会被清理
// `current-\d{8}.log` 由于分割时间精度不同也不会被清理
// 按天分割，日志最多保留7天。
let appender = FileAppender::rotate_with_expire("./current.log", Period::Day, Duration::days(7));
let logger = ftlog::builder()
    .root(appender)
    .build()
    .unwrap();
logger.init().unwrap();
```

### 日志发送到 UDP 服务器

将日志发送到 UDP 协议的日志机集中收集

```rust
// 目标服务监听的地址
let target_addr = "127.0.0.1:8080";
let udp_appender = UdpAppender::builder()
    .target(target_addr.parse().unwrap())
    .build();
ftlog::builder().root(udp_appender).try_init().unwrap();
```

## 可选功能
- **tsc**
  使用TSC寄存器作为时钟源，实现同等时间精度下更快地获取时间戳。

  注意，启用TSC必须满足以下条件：
  1. CPU不能变频
  1. 必须在 **x86架构** 的CPU上，目前只支持 **Linux** 系统。否则会启用备用方案，牺牲时间精度换取速度

## 性能评测

> Rust：1.67.0-nightly

//! |                                                   |  message type | Apple M1 Pro, 3.2GHz  | AMD EPYC 7T83, 3.2GHz |
//! | ------------------------------------------------- | ------------- | --------------------- | --------------------- |
//! | `ftlog`                                           | static string |   75 ns/iter    | 385 ns/iter    |
//! | `ftlog`                                           | with i32      |   106 ns/iter   | 491 ns/iter    |
//! | `env_logger` <br/> output to file                 | static string | 1,674 ns/iter  | 1,142 ns/iter   |
//! | `env_logger` <br/> output to file                 | with i32      | 1,681 ns/iter   | 1,179 ns/iter   |
//! | `env_logger` <br/> output to file with `BufWriter`| static string | 279 ns/iter     | 550 ns/iter     |
//! | `env_logger` <br/> output to file with `BufWriter`| with i32      | 278 ns/iter     | 565 ns/iter     |

License: MIT OR Apache-2.0
