use log::LevelFilter;
use simple_logger::SimpleLogger;

pub fn init() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug);
}
