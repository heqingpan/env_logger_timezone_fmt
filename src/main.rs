use env_logger::{Env, TimestampPrecision};
use env_logger_timezone_fmt::{TimeZoneFormat, TimeZoneFormatEnv};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    println!("hello, world!");
    // system local timezone
    //let timezone_fmt = Arc::new(TimeZoneFormatEnv::default());
    // system local timezone
    //let timezone_fmt = Arc::new(TimeZoneFormatEnv::new(None,Some(TimestampPrecision::Millis)));
    // GMT+8
    let timezone_fmt = Arc::new(TimeZoneFormatEnv::new(Some(8*60*60),Some(TimestampPrecision::Millis)));
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(move |buf, record| TimeZoneFormat::new(buf, &timezone_fmt).write(record))
        .init();
    log::info!("1");
    std::thread::sleep(Duration::from_millis(1000));
    log::info!("2");
    std::thread::sleep(Duration::from_millis(1000));
    log::info!("3");
}
