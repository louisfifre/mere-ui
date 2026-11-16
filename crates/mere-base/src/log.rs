//! The one hub every runtime observation goes through. Sinks plug onto it; the hub keeps nothing back and a
//! sink decides what it shows. Wire the sinks first thing in `main`, before anything that can observe.

use std::fmt;
use std::sync::Mutex;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct Record<'a> {
    pub level: Level,
    /// Which part of the program speaks, as `crate::module`.
    pub source: &'a str,
    pub message: fmt::Arguments<'a>,
}

type Sink = Box<dyn Fn(&Record) + Send + Sync>;
static SINKS: Mutex<Vec<Sink>> = Mutex::new(Vec::new());

pub fn add_sink(sink: impl Fn(&Record) + Send + Sync + 'static) {
    SINKS.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(sink));
}

pub fn emit(level: Level, source: &str, message: fmt::Arguments) {
    let record = Record { level, source, message };
    for sink in SINKS.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        sink(&record);
    }
}

/// A sink writing one line a record to standard error. A program without a console loses them, so a
/// windowed program plugs a file sink instead.
pub fn stderr_sink(min: Level) -> impl Fn(&Record) + Send + Sync {
    move |r: &Record| {
        if r.level >= min {
            eprintln!("{:?} {}: {}", r.level, r.source, r.message);
        }
    }
}

#[macro_export]
macro_rules! observe {
    ($level:ident, $($arg:tt)+) => {
        $crate::log::emit($crate::log::Level::$level, module_path!(), format_args!($($arg)+))
    };
}
