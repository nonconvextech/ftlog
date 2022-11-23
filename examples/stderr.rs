use std::io::stderr;

use ftlog::info;

fn main() {
    let builder = ftlog::builder().root(stderr()).build().unwrap();
    builder.init().unwrap();

    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        info!(limit=3000; "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log::logger().flush(); // force flush, otherwise log might be incomplete
    std::thread::sleep(std::time::Duration::from_secs(1));
}
