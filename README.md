# ftlog

普通的日志库受到磁盘io和系统pipe影响，单线程顺序写入单条速度大概要2500ns（SSD），如果碰到io抖动或者慢磁盘，日志会是低延时交易的主要瓶颈。
本库先把日志send到channel，再启动后台单独线程recv并且磁盘写入，测试速度在300ns左右。

本代码修改自`fastlog`以及官方的`log`库。

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
ftlog = "0.1.0"
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
use ftlog::*;

// 完整用法
// 配置logger
let logger = LogBuilder::new()
    //这里可以定义自己的格式，时间格式暂时不可以自定义
    .format(format)
    // a) 这里可以配置输出到文件
    .file(PathBuf::from("./current.log"))
    // b) 这里可以配置输出到文件，并且按指定间隔分割。这里导出的按天分割日志文件如current-20221024.log
    // 配置为按分钟分割时导出的日志文件如current-20221024T1428.log
    .file_split(PathBuf::from("./current.log"), Period::Day)
    // 如果既不配置输出文件 a)， 也不配置按指定间隔分割文件 b)，则默认输出到stderr
    // a) 和 b) 互斥，写在后面的生效，比如这里就是file_split生效
    .max_log_level(LevelFilter::Info)
    .build()
    .expect("logger build failed");
// 初始化
logger.init().expect("set logger failed");
```

#### ftlog与log

ftlog与rust的log生态不兼容，建议删除掉原来的日志库。特别是**不要让两个日志库导出到同一个地方**，否则两个日志生态会同时打印，导致日志不可读。

删除原有的log库，比如：
```toml
# log = "*"
# log4rs = "*"
# simple-logging = "*"
# env_logger = "*"
```

### 用法

```rust
trace!("Hello world!");
debug!("Hello world!");
info!("Hello world!");
warn!("Hello world!");
error!("Hello world!");
```

在main最后加入flush，否则在程序结束时未写入的日志会丢失：
```rust
ftlog::logger().flush();
```

参见 `examples/ftlog.rs`

#### 带间隔的日志
本库支持受限写入的功能。

```rust
info!(limit: 3000, "limit running{} !", i);
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
use ftlog::writer::file_split::Period;

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


### 其他库日志不输出的问题
ftlog为了防止日志输出太多卡住服务器，特意开发了limit: 3000的功能，
这个功能要改一个结构体导致跟官方的log库不兼容。

为了让用了官方`log::info!`的代码也可以输出日志（通常是引用的其他库的代码日志），需要在项目中配置logging，比如使用`simple-logging`, `env-logger`, `log4rs`等。

这里以`simple-logging`为例, `Cargo.toml` 加上
```toml
log = "*"
simple-logging = "*"
```

main函数初始化加上这一行代码，即可以把官方标准log::info!的日志输出到stderr
```rust
simple_logging::log_to_stderr(log::LevelFilter::Info);
```

建议指定crate的版本，并配置ftlog和rust标准log库输出到不同地方（如打印到两个不同文件），否则两个库可能同时写入导致日志错乱。
