use ftlog::{
    appender::{file::Period, FileAppender},
    info,
};
use log::LevelFilter;
use time::Duration;
fn init() {
    // Rotate every day, clean stale logs that were modified 7 days ago on each rotation
    let writer = FileAppender::rotate_with_expire("./current.log", Period::Day, Duration::weeks(1));
    let logger = ftlog::Builder::new()
        // global max log level
        .max_log_level(LevelFilter::Info)
        // define root appender, pass None would write to stderr
        .root(writer)
        // write logs in ftlog::appender to "./ftlog-appender.log" instead of "./current.log"
        .filter("ftlog::appender", "ftlog-appender", LevelFilter::Error)
        .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
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
