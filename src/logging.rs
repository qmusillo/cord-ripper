use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{debug, trace};

/// Log levels
pub const TRACE: usize = 0;
pub const DEBUG: usize = 1;
pub const INFO: usize = 2;
pub const WARN: usize = 3;
pub const ERROR: usize = 4;

/// Global log level
static LOG_LEVEL: AtomicUsize = AtomicUsize::new(INFO);

/// Set the log level dynamically
pub fn set_log_level(level: usize) {
    LOG_LEVEL.store(level, Ordering::Relaxed);
    debug!("Log level set to: {}", level);
}

/// Get the current log level
pub fn current_log_level() -> usize {
    LOG_LEVEL.load(Ordering::Relaxed)
}

/// Convert a log level string to its corresponding usize value
pub fn log_level_from_str(level: &str) -> Option<usize> {
    let level_usize = match level.to_lowercase().as_str() {
        "trace" => Some(TRACE),
        "debug" => Some(DEBUG),
        "info" => Some(INFO),
        "warn" | "warning" => Some(WARN),
        "error" => Some(ERROR),
        _ => None,
    };
    trace!("Got log level {:?} from string", level_usize);
    level_usize
}

#[macro_export]
macro_rules! log {
    ($level:expr, $color:expr, $tag:expr, $($arg:tt)*) => {
        if $level >= $crate::current_log_level() {
            println!(concat!("\x1b[", $color, "m", $tag, "\x1b[0m {}"), format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log!($crate::TRACE, "35", "[TRACE]", $($arg)*) // Magenta
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log!($crate::DEBUG, "34", "[DEBUG]", $($arg)*) // Blue
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log!($crate::INFO, "32", "[INFO]", $($arg)*) // Green
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log!($crate::WARN, "33", "[WARNING]", $($arg)*) // Yellow
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log!($crate::ERROR, "31", "[ERROR]", $($arg)*) // Red
    };
}
