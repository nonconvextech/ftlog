#![feature(test)]
extern crate test;
use std::hint::black_box;

use ftlog::{FtLogFormat, FtLogFormatter, Record};

#[bench]
fn format_value(b: &mut test::Bencher) {
    let record = Record::builder()
        .level(log::Level::Info)
        .file_static(Some("benches/string-format.rs"))
        .line(Some(29))
        .build();
    b.iter(|| {
        black_box(FtLogFormatter.msg(&record));
    });
}

#[bench]
fn format_string(b: &mut test::Bencher) {
    let format = Box::new(|record: &Record| {
        format!(
            "{} {}/{}:{} {}",
            record.level(),
            std::thread::current().name().unwrap_or_default(),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
            record.args()
        )
    });

    let record = Record::builder()
        .level(log::Level::Info)
        .file_static(Some("benches/string-format.rs"))
        .line(Some(29))
        .build();

    b.iter(|| {
        black_box(format(&record));
    });
}
