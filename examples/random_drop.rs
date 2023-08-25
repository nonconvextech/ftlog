use log::info;

fn main() {
    ftlog::builder().try_init().unwrap();

    // both `random_drop` and `drop` are recognized
    for i in 0..10 {
        info!(random_drop=0.0f32;"Always log: {}", i);
        info!(drop=1.0f32; "Always log: {}", i);
        info!(random_drop=0.9f32; "Randomly drop 90% of log calls: {}", i);
    }
    log::logger().flush();
}
