use ftlog::{
    appender::{file::Period, ChainAppenders, FileAppender},
    info, LoggerGuard,
};
use log::LevelFilter;
use time::Duration;

fn init() -> LoggerGuard {
    // Rotate every day, clean stale logs that were modified 7 days ago on each rotation
    let writer = FileAppender::builder()
        .path("./current.log")
        .rotate(Period::Minute)
        .expire(Duration::minutes(2))
        .build();
    ftlog::Builder::new()
        // global max log level
        .max_log_level(LevelFilter::Info)
        // define root appender, pass None would write to stderr
        .root(ChainAppenders::new(vec![
            Box::new(writer),
            Box::new(std::io::stdout()),
        ]))
        // write logs in ftlog::appender to "./ftlog-appender.log" instead of "./current.log"
        .filter_with(
            |_msg, level, target| target == "ftlog::appender" && level == LevelFilter::Error,
            "ftlog-appender",
        )
        .appender("ftlog-appender", FileAppender::new("ftlog-appender.log"))
        .try_init()
        .expect("logger build or set failed")
}

fn main() {
    let _guard = init();
    info!("Hello, world!");
    for i in 0..300 {
        info!("running {}!", i);
        info!(limit=3000i64; "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
