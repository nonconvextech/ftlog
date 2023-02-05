use std::time::{Duration, Instant};

const MIN_BENCH_TIME: u64 = 2000;
const MESSAGES: usize = 100_000;

macro_rules! run {
    ($name: expr, $code: expr) => {
        let mut sum_elapsed = Duration::default();
        let mut count = 0u32;
        loop {
            let now = Instant::now();
            for _ in 1..=MESSAGES {
                $code
            }
            let elapsed = now.elapsed();
            ftlog::logger().flush();

            sum_elapsed += elapsed;
            count += 1;
            if sum_elapsed >= Duration::from_millis(MIN_BENCH_TIME) && count > 10 {
                break;
            }
        }
        println!(
            "{},{}ns,{}/s",
            $name,
            (sum_elapsed / count / MESSAGES as u32).as_nanos(),
            ((count as f64 * MESSAGES as f64) / sum_elapsed.as_secs_f64()).round()
        );
    };
}
fn main() {
    ftlog::Builder::new()
        .root(std::io::sink())
        .bounded(100_000, false)
        // .unbounded()
        .build()
        .unwrap()
        .init()
        .unwrap();
    run!("static string", {
        ftlog::info!("ftlog message");
    });
    run!("with i32", {
        ftlog::info!("ftlog: {}", 0i32);
    });
    run!("limit with i32", {
        ftlog::info!(limit=3; "ftlog message");
    });
}
