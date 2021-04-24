use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::cell::RefCell;
use std::sync::Arc;
use std::io::Write;
use std::borrow::BorrowMut;

struct Logger(Arc<File>);

impl log::Log for Logger {
    fn enabled(&self, meta: &Metadata<'_>) -> bool {
        meta.level() <= Level::Debug
    }

    fn log(&self, record: &Record<'_>) {
        if !record.module_path().map(|mp| mp.contains("launchk")).unwrap_or(false) {
            return;
        }

        let statement = format!("{} - {}\n", record.level(), record.args());
        let Self(arc) = self;
        let mut file = &**arc;
        file.write(statement.as_bytes()).expect("Must write statement");
    }

    fn flush(&self) {
        let Self(arc) = self;
        let mut file = &**arc;
        file.flush().expect("Must flush to file");
    }
}

pub fn bind() -> Result<(), SetLoggerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Must get ts");
    let file = File::create(format!("launchk-debug-{}.txt", now.as_secs())).expect("Must open file");
    log::set_boxed_logger(Box::new(Logger(Arc::new(file))))
        .and_then(|()| Ok(log::set_max_level(LevelFilter::Debug)))
}