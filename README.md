# ftlog

[![Build Status](https://github.com/nonconvextech/ftlog/workflows/CI%20%28Linux%29/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
![License](https://img.shields.io/crates/l/ftlog.svg)
[![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
[![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)

普通的日志库受到磁盘io和系统pipe影响，单线程顺序写入单条速度大概要2500ns（SSD），如果碰到io抖动或者慢磁盘，日志会是低延时交易的主要瓶颈。
本库先把日志send到channel，再启动后台单独线程recv并且磁盘写入，测试速度在300ns左右。

CAUTION: this crate use `unchecked_math` unstable feature and `unsafe` code. Only use this crate in rust `nightly` channel.

#### 日志格式
```plain
2022-04-08 19:20:48.190+08 298ms <<这里是日志的延时，正常这个数为0，如果太大说明channel堵塞了 INFO main/src/ftlog.rs:14 14575
```

```plain
2022-04-10 21:27:15.996+08 0ms 2 <<这表示丢弃了2条日志，说明使用了limit语法 INFO main/src/main.rs:29 limit running3 !
```

### 配置
加入引用
```toml
ftlog = "0.2.0"
log = "0.4.17"
```

在main开头加入配置：
```rust
use ftlog::*;

// 最简配置用法，使用默认配置，输出到stderr
LogBuilder::new().build().unwrap().init().unwrap();

```

默认的配置会把日志输出到stderr，程序启动的时候要执行bin/yourjob &>> logs/server.log来把日志定向到文件。（注意shell要用/bin/bash不要用/bin/sh。/bin/sh在ubuntu下有问题，不标准)

也可以直接输出到固定文件，完整的配置用法如下:

```rust
use ftlog::{LogBuilder, writer::file_split::Period, LevelFilter};

// 完整用法
// 配置logger
let logger = LogBuilder::new()
    //这里可以定义自己的格式，时间格式暂时不可以自定义
    // .format(format)
    // a) 这里可以配置输出到文件
    .file(std::path::PathBuf::from("./current.log"))
    // b) 这里可以配置输出到文件，并且按指定间隔分割。这里导出的按天分割日志文件如current-20221024.log
    // 配置为按分钟分割时导出的日志文件如current-20221024T1428.log
    .file_split(std::path::PathBuf::from("./current.log"), Period::Day)
    // 如果既不配置输出文件 a)， 也不配置按指定间隔分割文件 b)，则默认输出到stderr
    // a) 和 b) 互斥，写在后面的生效，比如这里就是file_split生效
    .max_log_level(LevelFilter::Info)
    .build()
    .expect("logger build failed");
// 初始化
logger.init().expect("set logger failed");
```

### 用法

```rust, ignore
trace!("Hello world!");
debug!("Hello world!");
info!("Hello world!");
warn!("Hello world!");
error!("Hello world!");
```rust

在main最后加入flush，否则在程序结束时未写入的日志会丢失：
```rust, ignore
ftlog::logger().flush();
```

参见 `examples/ftlog.rs`

#### 带间隔的日志
本库支持受限写入的功能。

```rust
info!(limit=3000; "limit running{} !", 1);
```
上面这一行日志会有最小3000毫秒的间隔，也就是最多3000ms一条。

不同代码行的日志间隔是互相独立的，互不影响。

后台线程以(模块、文件名、代码行)为key来记录上次日志打印时间，小于间隔则跳过。

#### 日志分割
本库支持日志按时间分割，自动适配本地时区，可选分割时间精度如下：

- 分钟 `Period::Minute`
- 小时 `Period::Hour`
- 日 `Period::Day`
- 月 `Period::Month`
- 年 `Period::Year`

```rust
use std::path::PathBuf;
use ftlog::{LogBuilder, writer::file_split::Period};

let logger = LogBuilder::new()
    .file_split(PathBuf::from("./current.log"), Period::Minute)
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
current-20221026T1352.log
current-20221026T1353.log
# 按小时分割
current-20221026T13.log
current-20221026T14.log
# 按日分割
current-20221026.log
current-20221027.log
# 按月分割
current-202210.log
current-202211.log
# 按年分割
current-2022.log
current-2023.log
# 设置的保存文件名无拓展时(如"./log")，则直接在文件名末尾加时间
log-20221026T1351
log-20221026T1352
log-20221026T1353
```

License: MIT OR Apache-2.0
