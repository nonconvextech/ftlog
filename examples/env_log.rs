use ftlog::{appender::FileAppender, LoggerGuard};
use log::{info, LevelFilter};

fn init() -> LoggerGuard {
    // Rotate every day, clean stale logs that were modified 7 days ago on each rotation
    let writer = FileAppender::builder()
    .path("./env_log.log")
    .build();
    ftlog::Builder::new()
        // global max log level
        .max_log_level(LevelFilter::Info)
        .use_env_filter()
        // define root appender, pass None would write to stderr
        .root(writer)
        // write logs in ftlog::appender to "./ftlog-appender.log" instead of "./current.log"
        .try_init()
        .expect("logger build or set failed")
}

fn main() {
    // RUST_LOG=env_log cargo run --example env_log or RUST_LOG=info cargo run --example env_log will show log lines
    let _guard = init();
    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        info!(limit=3000i64; "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    std::thread::sleep(std::time::Duration::from_secs(5));
}
