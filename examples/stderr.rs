use ftlog::info;

fn main() {
    ftlog::builder().try_init().unwrap();

    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        info!(limit=3000i64; "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log::logger().flush();
}
