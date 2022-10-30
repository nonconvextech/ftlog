use std::path::PathBuf;

use ftlog::{writer::file_split::Period, *};
// use std::path::PathBuf;
// use time::{at, get_time, Timespec};
fn init() {
    /*
    let format = |record: &Record| {
        format!(
            "{}{}/{}:{}[{}] {}",
            std::thread::current().name().unwrap_or_default(),
            record.module_path().unwrap_or(""),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
            record.level(),
            record.args()
        )
    };
    正常不需要改格式，这段代码的格式和标准的是不一样的，为了性能，时间格式不可以改
     */
    let logger = LogBuilder::new()
        // .format(format)//这里可以定义自己的格式，时间格式暂时不可以自定义
        // a) 这里可以配置输出到文件
        // .file(PathBuf::from("./current.log"))
        // b) 这里可以配置输出到文件，并且按指定间隔分割，实际导出的日志文件如current-20221024T1420.log
        // 如果不配置输出文件a)也不配置按天分割文件b)，则默认输出到stderr
        .file_split(PathBuf::from("./current.log"), Period::Minute)
        .max_log_level(LevelFilter::Info)
        .build()
        .expect("logger build failed");
    logger.init().expect("set logger failed");

    //    simple_logging::log_to_stderr(log::LevelFilter::Info);//加上这一行可以输出其他库的日志，也就是标准库的日志
}
/**
* main函数在本地ide可以直接点击执行，调试逻辑
 */
fn main() {
    init();
    info!("Hello, world!");
    for i in 0..120 {
        info!("running {}!", i);
        info!(limit: 3000, "limit running{} !", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    ftlog::logger().flush(); //这个可以强行flush，也可以不调用，但是最后程序结束的时候可能丢日志
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/*
上述代码输出：

2022-04-11 15:08:19.847+08 0ms INFO main/src/main.rs:25 Hello, world!
2022-04-11 15:08:19.847+08 0ms INFO main/src/main.rs:28 running 0!
2022-04-11 15:08:19.847+08 0ms 0 INFO main/src/main.rs:29 limit running0 !
2022-04-11 15:08:20.849+08 0ms INFO main/src/main.rs:28 running 1!
2022-04-11 15:08:21.852+08 0ms INFO main/src/main.rs:28 running 2!
2022-04-11 15:08:22.857+08 0ms INFO main/src/main.rs:28 running 3!
2022-04-11 15:08:22.857+08 0ms 2 INFO main/src/main.rs:29 limit running3 !
2022-04-11 15:08:23.862+08 0ms INFO main/src/main.rs:28 running 4!
2022-04-11 15:08:24.864+08 0ms INFO main/src/main.rs:28 running 5!

 */
