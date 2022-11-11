#![feature(test)]
extern crate test;
use ftlog::{appender::FileAppender, FtLogFormat, Record};

struct StringFormatter;
impl FtLogFormat for StringFormatter {
    fn msg(&self, record: &Record) -> Box<dyn Send + Sync + std::fmt::Display> {
        Box::new(format!(
            "{} {}/{}:{} {}",
            record.level(),
            std::thread::current().name().unwrap_or_default(),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
            record.args()
        ))
    }
}

#[bench]
fn static_string(b: &mut test::Bencher) {
    ftlog::Builder::new()
        .format(StringFormatter)
        .root(FileAppender::new("bench.log"))
        .build()
        .unwrap()
        .init()
        .unwrap();
    b.iter(|| {
        ftlog::info!("ftlog message");
    });
}

#[bench]
fn with_i32(b: &mut test::Bencher) {
    let i = 0;
    b.iter(|| {
        ftlog::info!("ftlog: {}", i);
    });
}
