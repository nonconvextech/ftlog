use ftlog::{
    appender::{file::Period, FileAppender},
    info, LoggerGuard,
};
use log::LevelFilter;
use time::Duration;

fn init() -> LoggerGuard {
    // Rotate every day, clean stale logs that were modified 7 days ago on each rotation
    let writer = FileAppender::builder()
        .path("./current.log")
        .rotate(Period::Minute)
        .expire(Duration::minutes(4))
        .build();
    ftlog::Builder::new()
        // global max log level
        .max_log_level(LevelFilter::Info)
        // define root appender, pass None would write to stderr
        .root(writer)
        // write logs in ftlog::appender to "./ftlog-appender.log" instead of "./current.log"
        .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
        .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
        .try_init()
        .expect("logger build or set failed")
}

fn main() {
    let _guard = init();
    // info!("Hello, world!");
    // for i in 0..120 {
    //     info!("running {}!", i);
    //     info!(limit=3000i64; "limit running{} !", i);
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    // }
    std::thread::sleep(std::time::Duration::from_secs(5));
}
