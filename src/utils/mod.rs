pub mod encoding;

use std::fmt::Display;
use std::time::SystemTime;

#[allow(dead_code)]
pub enum Level {
    Info,
    Warn,
    Error,
    Debug,
}

pub fn log(level: Level, component: &str, message: impl Display) {
    let now = SystemTime::now();
    let datetime: chrono::DateTime<chrono::Local> = now.into();
    let timestamp = datetime.format("%H:%M:%S").to_string();

    let (color, label) = match level {
        Level::Info => ("\x1b[32m", "INFO"),
        Level::Warn => ("\x1b[33m", "WARN"),
        Level::Error => ("\x1b[31m", "ERROR"),
        Level::Debug => ("\x1b[35m", "DEBUG"),
    };

    println!("\x1b[90m[{}]\x1b[0m {}{}\x1b[0m \x1b[90m[{}]\x1b[0m {}", timestamp, color, label, component, message);
}