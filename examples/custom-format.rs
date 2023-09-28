use std::fmt::Display;

use ftlog::{
    appender::{file::Period, FileAppender},
    info, FtLogFormat,
};
use log::{Level, LevelFilter, Record};
use time::Duration;
fn init() {
    // Custom log style.

    // A formatter defines how to build a message.
    // Since Formatting message into string can slow down the log macro call,
    // the idomatic way is to send required field as is to log thread, and build
    // message in log thread.
    struct MyFormatter;
    impl FtLogFormat for MyFormatter {
        fn msg(&self, record: &Record) -> Box<dyn Send + Sync + std::fmt::Display> {
            Box::new(Msg {
                level: record.level(),
                thread: std::thread::current().name().map(|n| n.to_string()),
                file: record.file_static(),
                line: record.line(),
                args: format!("{}", record.args()),
                module_path: record.module_path_static(),
            })
        }
    }

    // Store necessary field, define how to build into string with `Display` trait.
    struct Msg {
        level: Level,
        thread: Option<String>,
        file: Option<&'static str>,
        line: Option<u32>,
        args: String,
        module_path: Option<&'static str>,
    }

    impl Display for Msg {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&format!(
                "{}@{}||{}:{}[{}] {}",
                self.thread.as_ref().map(|x| x.as_str()).unwrap_or(""),
                self.module_path.unwrap_or(""),
                self.file.unwrap_or(""),
                self.line.unwrap_or(0),
                self.level,
                self.args
            ))
        }
    }

    let time_format = time::format_description::parse_owned::<1>(
        "[year]/[month]/[day] [hour]:[minute]:[second].[subsecond digits:6]",
    )
    .unwrap();
    ftlog::Builder::new()
        // use our own format
        .format(MyFormatter)
        // use our own time format
        .time_format(time_format)
        // global max log level
        .max_log_level(LevelFilter::Info)
        // define root appender, pass None would write to stderr
        .root(
            FileAppender::builder()
                .path("./current.log")
                .rotate(Period::Day)
                .expire(Duration::days(7))
                .build(),
        )
        // ---------- configure additional filter ----------
        // write to "ftlog-appender" appender, with different level filter
        .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
        // write to root appender, but with different level filter
        .filter("ftlog", None, LevelFilter::Trace)
        // write to "ftlog" appender, with default level filter
        .filter("ftlog::appender::file", "ftlog", None)
        // ----------  configure additional appender ----------
        // new appender
        .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
        // new appender, rotate to new file every Day
        .appender("ftlog", FileAppender::rotate("ftlog.log", Period::Day))
        .try_init()
        .expect("logger build or set failed");
}

fn main() {
    init();
    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        info!(limit=3000i64; "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log::logger().flush();
}

/*
Output:

2022/11/11 13:53:13.933123 0ms logger@ftlog||src/lib.rs:439[WARN] Logs with level more verbose than INFO will be ignored in `ftlog`
2022/11/11 13:53:13.933123 0ms main@complex||examples/complex.rs:83[INFO] Hello, world!
2022/11/11 13:53:13.934123 1ms main@complex||examples/complex.rs:85[INFO] running 0!
2022/11/11 13:53:13.934123 3ms 0 main@complex||examples/complex.rs:86[INFO] limit running0 !
2022/11/11 13:53:13.934123 3ms logger@ftlog::appender::file||src/appender/file.rs:255[INFO] Log file deleted: current-20221111T1352.log
2022/11/11 13:53:14.939123 0ms main@complex||examples/complex.rs:85[INFO] running 1!
2022/11/11 13:53:15.939123 0ms main@complex||examples/complex.rs:85[INFO] running 2!
2022/11/11 13:53:16.943123 0ms main@complex||examples/complex.rs:85[INFO] running 3!
2022/11/11 13:53:16.943123 0ms 2 main@complex||examples/complex.rs:86[INFO] limit running3 !
2022/11/11 13:53:17.945123 0ms main@complex||examples/complex.rs:85[INFO] running 4!
2022/11/11 13:53:18.946123 0ms main@complex||examples/complex.rs:85[INFO] running 5!
2022/11/11 13:53:19.951123 0ms main@complex||examples/complex.rs:85[INFO] running 6!
2022/11/11 13:53:19.951123 0ms 2 main@complex||examples/complex.rs:86[INFO] limit running6 !
2022/11/11 13:53:20.956123 0ms main@complex||examples/complex.rs:85[INFO] running 7!
2022/11/11 13:53:21.961123 0ms main@complex||examples/complex.rs:85[INFO] running 8!
2022/11/11 13:53:22.966123 0ms main@complex||examples/complex.rs:85[INFO] running 9!
2022/11/11 13:53:22.966123 4ms 2 main@complex||examples/complex.rs:86[INFO] limit running9 !


Default style example:
2022-04-11 15:08:19.847+08 0ms INFO main@src/main.rs:25 Hello, world!
2022-04-11 15:08:19.847+08 0ms INFO main@src/main.rs:28 running 0!
2022-04-11 15:08:19.847+08 0ms 0 INFO main@src/main.rs:29 limit running0 !
2022-04-11 15:08:20.849+08 0ms INFO main@src/main.rs:28 running 1!
2022-04-11 15:08:21.852+08 0ms INFO main@src/main.rs:28 running 2!
2022-04-11 15:08:22.857+08 0ms INFO main@src/main.rs:28 running 3!
2022-04-11 15:08:22.857+08 0ms 2 INFO main@src/main.rs:29 limit running3 !
2022-04-11 15:08:23.862+08 0ms INFO main@src/main.rs:28 running 4!
2022-04-11 15:08:24.864+08 0ms INFO main@src/main.rs:28 running 5!
 */
