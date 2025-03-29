use std::{
    fs::{read_dir, remove_file},
    time::Instant,
};

use ftlog::appender::{Duration, FileAppender, Period};

pub fn setup_with_closures() {
    let logger = ftlog::Builder::new()
        .bounded(10000, true)
        .root(FileAppender::new("./root.log"))
        .filter_with(|_msg, _level, target| target == "rotate", "rotate")
        .appender("rotate", FileAppender::rotate("rotate.log", Period::Minute))
        .filter_with(|_msg, _level, target| target == "expire", "expire")
        .appender(
            "expire",
            FileAppender::rotate_with_expire("expire.log", Period::Day, Duration::days(7)),
        )
        .build()
        .expect("logger build failed");
    logger.init().expect("set logger failed");
}
pub fn setup_with_filter() {
    let logger = ftlog::Builder::new()
        .bounded(10000, true)
        .root(FileAppender::new("./root.log"))
        // .utc()
        .filter("rotate", "rotate", None)
        .appender("rotate", FileAppender::rotate("rotate.log", Period::Minute))
        .filter("expire", "expire", None)
        .appender(
            "expire",
            FileAppender::rotate_with_expire("expire.log", Period::Day, Duration::days(7)),
        )
        .build()
        .expect("logger build failed");
    logger.init().expect("set logger failed");
}

fn clean(dir: &str) {
    for file in read_dir(dir)
        .unwrap()
        .filter_map(|x| x.ok())
        .filter(|f| f.file_type().unwrap().is_file())
    {
        if file
            .path()
            .extension()
            .map(|x| x.to_string_lossy())
            .unwrap_or_default()
            == "log"
        {
            remove_file(file.path()).unwrap();
        }
    }
}
#[test]
fn test_speed_local() {
    // ~80MB
    setup_with_filter();
    let elapsed1 = {
        // file
        let now = Instant::now();
        for i in 1..=1_000_000 {
            ftlog::info!("file log {}", i);
        }
        ftlog::logger().flush();
        let elapsed = now.elapsed();
        println!("File elapsed: {}s", elapsed.as_secs_f64());
        elapsed
    };

    let elapsed2 = {
        // file with rotate
        let now = Instant::now();
        for i in 1..=1_000_000 {
            ftlog::info!(target:"rotate", "file log {}", i);
        }
        ftlog::logger().flush();
        let elapsed = now.elapsed();
        println!("Rotate file elapsed: {}s", elapsed.as_secs_f64());
        elapsed
    };

    let elapsed3 = {
        // file with rotate with expire
        let now = Instant::now();
        for i in 1..=1_000_000 {
            ftlog::info!(target:"expire", "file log {}", i);
        }
        ftlog::logger().flush();
        let elapsed = now.elapsed();
        println!(
            "Rotate file with expire elapsed: {}s",
            elapsed.as_secs_f64()
        );
        elapsed
    };
    clean("./");
    assert!(elapsed1.as_secs() < 8);
    assert!(elapsed2.as_secs() < 8);
    assert!(elapsed3.as_secs() < 8);
}

#[test]
fn test_speed_local2() {
    // ~80MB
    setup_with_closures();
    let elapsed1 = {
        // file
        let now = Instant::now();
        for i in 1..=1_000_000 {
            ftlog::info!("file log {}", i);
        }
        ftlog::logger().flush();
        let elapsed = now.elapsed();
        println!("File elapsed: {}s", elapsed.as_secs_f64());
        elapsed
    };

    let elapsed2 = {
        // file with rotate
        let now = Instant::now();
        for i in 1..=1_000_000 {
            ftlog::info!(target:"rotate", "file log {}", i);
        }
        ftlog::logger().flush();
        let elapsed = now.elapsed();
        println!("Rotate file elapsed: {}s", elapsed.as_secs_f64());
        elapsed
    };

    let elapsed3 = {
        // file with rotate with expire
        let now = Instant::now();
        for i in 1..=1_000_000 {
            ftlog::info!(target:"expire", "file log {}", i);
        }
        ftlog::logger().flush();
        let elapsed = now.elapsed();
        println!(
            "Rotate file with expire elapsed: {}s",
            elapsed.as_secs_f64()
        );
        elapsed
    };
    clean("./");
    assert!(elapsed1.as_secs() < 8);
    assert!(elapsed2.as_secs() < 8);
    assert!(elapsed3.as_secs() < 8);
}
