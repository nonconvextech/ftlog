use std::hint::black_box;
use std::time::{Duration, Instant};

use ftlog::{FtLogFormat, Record};

const MIN_BENCH_TIME: u64 = 2000;
const MESSAGES: usize = 100_000;

macro_rules! run {
    ($name: expr, $code: expr) => {
        let mut sum_elapsed = Duration::default();
        let mut count = 0u32;
        loop {
            let now = Instant::now();
            for _ in 1..=MESSAGES {
                black_box($code)
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
fn main() {
    ftlog::Builder::new()
        .root(std::io::sink())
        .format(StringFormatter)
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
        ftlog::info!(limit=3; "ftlog: {}", 0i32);
    });
}
