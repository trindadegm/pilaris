use log::{Level, Metadata, Record};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct PilarisLogger {
    enabled: AtomicUsize,
    level_int: AtomicUsize,
}

static PILARIS_LOGGER: PilarisLogger = PilarisLogger {
    enabled: AtomicUsize::new(0),
    level_int: AtomicUsize::new(0),
};

impl PilarisLogger {
    pub fn init(min_level: Level) {
        PilarisLogger::set_enabled(true);
        PilarisLogger::set_level(min_level);
        log::set_logger(&PILARIS_LOGGER).ok();
        log::set_max_level(log::LevelFilter::Trace)
    }

    pub fn set_enabled(enabled: bool) {
        PILARIS_LOGGER
            .enabled
            .store(enabled as usize, Ordering::Relaxed);
    }

    pub fn set_level(min_level: Level) {
        PILARIS_LOGGER
            .level_int
            .store(min_level as usize, Ordering::Relaxed);
    }

    pub fn min_level(&self) -> Level {
        let level_int = self.level_int.load(Ordering::Relaxed);
        match level_int {
            n if n == Level::Error as usize => Level::Error,
            n if n == Level::Warn as usize => Level::Warn,
            n if n == Level::Info as usize => Level::Info,
            n if n == Level::Debug as usize => Level::Debug,
            n if n == Level::Trace as usize => Level::Trace,
            _ => {
                eprintln!(
                    "Broken log level: {}. Pretending to be Error level.",
                    level_int
                );
                Level::Error
            }
        }
    }
}
impl log::Log for PilarisLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.enabled.load(Ordering::Relaxed) != 0 && metadata.level() < self.min_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}][{}:{}] {}",
                record.level(),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                record.args()
            );
        }
    }

    fn flush(&self) {
        use std::io::Write;
        std::io::stdout().lock().flush().ok();
    }
}
