# ftlog

[![Build Status](https://github.com/nonconvextech/ftlog/workflows/ftlog/badge.svg?branch=main)](https://github.com/nonconvextech/ftlog/actions)
![License](https://img.shields.io/crates/l/ftlog.svg)
[![Latest Version](https://img.shields.io/crates/v/ftlog.svg)](https://crates.io/crates/ftlog)
[![ftlog](https://docs.rs/ftlog/badge.svg)](https://docs.rs/ftlog)

Logging is affected by the disk IO and pipe system call.
Sequential log calls can be a bottleneck in scenarios where low
latency is critical (e.g., high-frequency trading).

`ftlog` mitigates this bottleneck by sending messages to a dedicated logger
thread and computing as little as possible in the main/worker thread.

`ftlog` can improve log performance in main/worker thread a few times over. See
performance for details.


## Usage

Add to your `Cargo.toml`:

```toml
ftlog = "0.2"
```

Configure and initialize ftlog at the start of your `main` function:
```rust
// ftlog re-export `log`'s macros, so no need to add `log` to dependencies
use ftlog::appender::FileAppender;
use ftlog::{debug, trace};
use log::{error, info, warn};

// minimal configuration with default setting

// When drops, the guard calls and waits `flush` to logger.
// With guard that share the lifetime of `main` fn, there is no need to manually call flush at the end of `main` fn.
let _guard = ftlog::builder().try_init().unwrap();

trace!("Hello world!");
debug!("Hello world!");
info!("Hello world!");
warn!("Hello world!");
error!("Hello world!");
```

A more complicated but feature rich usage:

```rust
use ftlog::{
    appender::{Duration, FileAppender, Period},
    FtLogFormatter, LevelFilter,
};

let time_format = time::format_description::parse_owned::<1>(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]",
)
.unwrap();
// configurate logger
let _guard = ftlog::builder()
    // global max log level
    .max_log_level(LevelFilter::Info)
    // custom timestamp format
    .time_format(time_format)
    // set global log formatter
    .format(FtLogFormatter)
    // use bounded channel to avoid large memory comsumption when overwhelmed with logs
    // Set `false` to tell ftlog to discard excessive logs.
    // Set `true` to block log call to wait for log thread.
    // here is the default settings
    .bounded(100_000, false) // .unbounded()
    // define root appender, pass anything that is Write and Send
    // omit `Builder::root` will write to stderr
    .root(
        FileAppender::builder()
            .path("./current.log")
            .rotate(Period::Day)
            .expire(Duration::days(7))
            .build(),
    )
    // timezone of log message timestamp, use local by default
    // .local_timezone()
    // or use fiexed timezone for better throughput, since retrieving timezone is a time consuming operation
    // this does not affect worker threads (that call log), but can boost log thread performance (higher throughput).
    .fixed_timezone(time::UtcOffset::current_local_offset().unwrap())
    // level filter for root appender
    .root_log_level(LevelFilter::Warn)
    // write logs in ftlog::appender to "./ftlog-appender.log" instead of "./current.log"
    .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
    .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
    .try_init()
    .expect("logger build or set failed");
```

See `./examples` for more (e.g. custom format).

### Default Log Format

> 2022-04-08 19:20:48.190+08 **298ms** INFO main [src/ftlog.rs:14] My log
> message

Here `298ms` denotes the latency between the call of the log (e.g.
`log::info!("msg")`) and the actual printing in log thread. Normally this is 0ms.

A large delay indicates that the log thread may be blocked by excessive log
messages.

> 2022-04-10 21:27:15.996+08 0ms **2** INFO main [src/main.rs:29] limit
> running3 !

The number **2** above indicates how many log messages were discarded.
Only shown if the frequency of logging for a single log call is limited (e.g.
`log::info!(limit=3000i64;"msg")`).

### Randomly drop log

Use `random_drop` or `drop` to specify the probability of randomly discarding logs.
No message is dropped by default.

```rust
log::info!(random_drop=0.1f32;"Random log 10% of log calls, keeps 90%");
log::info!(drop=0.99f32;"Random drop 99% of log calls, keeps 1%");
```

This can be helpful when formatting log message into string is too costly,

When both `random_drop` and `limit` is specified,
ftlog will limit logs after messages are randomly dropped.
```rust
log::info!(drop=0.99f32,limit=1000;
    "Drop 99% messages. The survived 1% messages are limit to at least 1000ms between adjacent log messages output"
);
```

### Custom timestamp format

`ftlog` relies on the `time` crate for the formatting of timestamp. To use custom time format,
first construct a valid time format description,
and then pass it to ftlog builder by `ftlog::time_format(&mut self)`.

In case an error occurs when formatting timestamp, `ftlog` will fallback to RFC3339 time format.

#### Example
```rust
let format = time::format_description::parse_owned::<1>(
    "[year]/[month]/[day] [hour]:[minute]:[second].[subsecond digits:6]",
)
.unwrap();
let _guard = ftlog::builder().time_format(format).try_init().unwrap();
log::info!("Log with custom timestamp format");
// Output:
// 2023/06/14 11:13:26.160840 0ms INFO main [main.rs:3] Log with custom timestamp format
```

### Log with interval

`ftlog` allows to limit the write frequency for individual log calls.

If the above line is called multiple times within 3000ms, then it is logged only
once, with an added number reflecting the number of discarded log messages.

Each log call ha an independent interval, so we can set different intervals
for different log calls. Internally, `ftlog` records the last print time by a
combination of (module name, file name, code line).

#### Example

```rust
info!(limit=3000i64; "limit running {}s !", 3);
```
The minimal interval of the the specific log call above is 3000ms.

```markdown
2022-04-10 21:27:10.996+08 0ms 0 INFO main [src/main.rs:29] limit running 3s !
2022-04-10 21:27:15.996+08 0ms 2 INFO main [src/main.rs:29] limit running 3s !
```
The number **2** above shows how many log messages is discarded since last log.

### Log rotation
`ftlog` supports log rotation in local timezone. The available rotation
periods are:

- minute `Period::Minute`
- hour `Period::Hour`
- day `Period::Day`
- month `Period::Month`
- year `Period::Year`

Log rotation is configured in `FileAppender`, and the timestamp is appended to
the end of the filename:

```rust
use ftlog::appender::{FileAppender, Period};

let logger = ftlog::builder()
    .root(
        FileAppender::builder()
            .path("./mylog.log")
            .rotate(Period::Minute)
            .build(),
    )
    .build()
    .unwrap();
let _guard = logger.init().unwrap();
```

If the log file is configured to be split by minutes,
the log file name has the format
`mylog-{MMMM}{YY}{DD}T{hh}{mm}.log`. When divided by days, the log file name is
something like `mylog-{MMMM}{YY}{DD}.log`.

Log filename examples:
```sh
$ ls
# by minute
current-20221026T1351.log
# by hour
current-20221026T13.log
# by day
current-20221026.log
# by month
current-202210.log
# by year
current-2022.log
# omitting extension (e.g. "./log") will add datetime to the end of log filename
log-20221026T1351
```

#### Clean outdated logs

With log rotation enabled, it is possible to clean outdated logs to free up
disk space with `FileAppender::rotate_with_expire` method or set `expire(Duration)`
when using builder.

`ftlog` first finds files generated by `ftlog` and cleans outdated logs by
last modified time. `ftlog` find generated logs by filename matched by file
stem and added datetime.

**ATTENTION**: Any files that matchs the pattern will be deleted.

```rust
use ftlog::{appender::{Period, FileAppender, Duration}};

// clean files named like `current-\d{8}T\d{4}.log`.
// files like `another-\d{8}T\d{4}.log` or `current-\d{8}T\d{4}` will not be deleted, since the filenames' stem do not match.
// files like `current-\d{8}.log` will remains either, since the rotation durations do not match.

// Rotate every day, clean stale logs that were modified 7 days ago on each rotation
let appender = FileAppender::rotate_with_expire("./current.log", Period::Day, Duration::days(7));
let logger = ftlog::builder()
    .root(appender)
    .build()
    .unwrap();
let _guard = logger.init().unwrap();
```

### Sending Logs to a UDP Server

Send logs to a logging server using the UDP protocol

```rust
//  Address that the target server is listening on
let target_addr = "127.0.0.1:8080";
let udp_appender = UdpAppender::builder()
    .target(target_addr.parse().unwrap())
    .build();
ftlog::builder().root(udp_appender).try_init().unwrap();
```

## Features
- **tsc**
  Use [TSC](https://en.wikipedia.org/wiki/Time_Stamp_Counter) for clock source for higher performance without
  accuracy loss.

  TSC offers the most accurate and cheapest way to access current time under certain condition:
  1. the CPU frequency must be constant
  1. must with CPU of x86/x86_64 architecture, since TSC is an x86/x86_64 specific register.
  1. never suspend

  The current feature further requires that the build target **MUST BE LINUX**. Otherwise it will fall back to
  a fast but much less accurate implementation.

## Timezone

For performance, timezone is detected once at logger buildup, and use it later in every
log message. This is partly due to timezone detetion is expensive, and partly to the unsafe
nature of underlying system call in multi-thread program in Linux.

It's also recommended to use UTC instead to further avoid timestamp convertion to timezone for every log message.

## Performance

> Rust：1.67.0-nightly

|                                                   |  message type | Apple M1 Pro, 3.2GHz  | AMD EPYC 7T83, 3.2GHz |
| ------------------------------------------------- | ------------- | --------------------- | --------------------- |
| `ftlog`                                           | static string |   75 ns/iter    | 385 ns/iter    |
| `ftlog`                                           | with i32      |   106 ns/iter   | 491 ns/iter    |
| `env_logger` <br/> output to file                 | static string | 1,674 ns/iter  | 1,142 ns/iter   |
| `env_logger` <br/> output to file                 | with i32      | 1,681 ns/iter   | 1,179 ns/iter   |
| `env_logger` <br/> output to file with `BufWriter`| static string | 279 ns/iter     | 550 ns/iter     |
| `env_logger` <br/> output to file with `BufWriter`| with i32      | 278 ns/iter     | 565 ns/iter     |

License: MIT OR Apache-2.0
