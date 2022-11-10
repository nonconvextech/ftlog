use ftlog::{
    appender::{file::Period, FileAppender},
    info, LogBuilder,
};
use log::{LevelFilter, Record};
use time::Duration;
fn init() {
    // we can modify log style. Datetime format is fiex for performance
    let format = |record: &Record| {
        format!(
            "{}{}/{}:{}[{}] {}",
            std::thread::current().name().unwrap_or_default(),
            record.module_path().unwrap_or(""),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
            record.level(),
            record.args()
        )
    };
    let logger = LogBuilder::new()
        .format(format) // define our own format
        .max_log_level(LevelFilter::Info)
        // define root appender, pass None would write to stderr
        .root(FileAppender::rotate_with_expire(
            "./current.log",
            Period::Minute,
            Duration::seconds(30),
        ))
        .build()
        .expect("logger build failed");
    logger.init().expect("set logger failed");
}
fn main() {
    init();
    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        info!(limit=3000; "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log::logger().flush(); // force flush, otherwise log might be incomplete
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/*
Output:

2022-10-30 22:21:27.878+08 0ms mainftlog/examples/ftlog.rs:37[INFO] Hello, world!
2022-10-30 22:21:27.878+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 0!
2022-10-30 22:21:27.878+08 0ms 0 mainftlog/examples/ftlog.rs:40[INFO] limit running0 !
2022-10-30 22:21:28.883+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 1!
2022-10-30 22:21:29.885+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 2!
2022-10-30 22:21:30.890+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 3!
2022-10-30 22:21:30.890+08 0ms 2 mainftlog/examples/ftlog.rs:40[INFO] limit running3 !
2022-10-30 22:21:31.895+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 4!
2022-10-30 22:21:32.900+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 5!
2022-10-30 22:21:33.905+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 6!
2022-10-30 22:21:33.905+08 0ms 2 mainftlog/examples/ftlog.rs:40[INFO] limit running6 !
2022-10-30 22:21:34.907+08 0ms mainftlog/examples/ftlog.rs:39[INFO] running 7!

Default style example:
2022-04-11 15:08:19.847+08 0ms INFO main/src/main.rs:25 Hello, world!
2022-04-11 15:08:19.847+08 0ms INFO main/src/main.rs:28 running 0!
2022-04-11 15:08:19.847+08 0ms 0 INFO main/src/main.rs:29 limit running0 !
2022-04-11 15:08:20.849+08 0ms INFO main/src/main.rs:28 running 1!
2022-04-11 15:08:21.852+08 0ms INFO main/src/main.rs:28 running 2!
2022-04-11 15:08:22.857+08 0ms INFO main/src/main.rs:28 running 3!
2022-04-11 15:08:22.857+08 0ms 2 INFO main/src/main.rs:29 limit running3 !
2022-04-11 15:08:23.862+08 0ms INFO main/src/main.rs:28 running 4!
2022-04-11 15:08:24.864+08 0ms INFO main/src/main.rs:28 running 5!
 */

// fn test {

//     ftlog::Logger::builder()
//     .root()
//     .build();

//  }
