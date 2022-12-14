#![feature(test)]
extern crate test;
use ftlog::appender::FileAppender;

#[bench]
fn static_string(b: &mut test::Bencher) {
    ftlog::Builder::new()
        .root(FileAppender::new("bench-lazy.log"))
        .bounded(10_000, false)
        .build()
        .unwrap()
        .init()
        .unwrap();
    b.iter(|| {
        ftlog::info!("ftlog message");
    });
    log::logger().flush();
}

#[bench]
fn with_i32(b: &mut test::Bencher) {
    let i = 0;
    b.iter(|| {
        ftlog::info!("ftlog: {}", i);
    });
    log::logger().flush();
}
