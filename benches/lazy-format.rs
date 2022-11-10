use criterion::{criterion_group, criterion_main, Criterion};
use ftlog::appender::FileAppender;

fn criterion_benchmark(c: &mut Criterion) {
    ftlog::LogBuilder::new()
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
