use std::time::SystemTime;

use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::LevelFilter;

pub fn init() {
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .trace(Color::BrightBlack);

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                // Color the info the level color, and the message bright white
                "{}[{} {} {}]\x1B[0m {}{}\x1B[0m",
                format_args!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str()),
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                format_args!("\x1B[{}m", Color::BrightWhite.to_fg_str()),
                message
            ))
        })
        // Only info from dependencies
        .level(LevelFilter::Info)
        // But everything for this crate (golem)
        .level_for("golem", LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .expect("logger should initialize");
}
