use criterion::{criterion_group, criterion_main, Criterion};
use ftlog::{appender::FileAppender, FtLogFormat, Record};

fn criterion_benchmark(c: &mut Criterion) {
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

    ftlog::LogBuilder::new()
        .format(StringFormatter)
        .root(FileAppender::new("bench.log"))
        .build()
        .unwrap()
        .init()
        .unwrap();

    c.bench_function("static-string", |b| {
        b.iter(|| {
            ftlog::info!("ftlog message");
        });
    });
    c.bench_function("with i32", |b| {
        let i = 0;
        b.iter(|| {
            ftlog::info!("ftlog: {}", i);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
