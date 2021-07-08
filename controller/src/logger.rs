use chrono::Utc;
use std::sync::mpsc::Receiver;

use colored::{ColoredString, Colorize};

#[allow(dead_code)]
pub enum LogType {
    Error,
    Log,
    Warn,
    Debug,
}

pub struct Logger {
    receiver: Receiver<LoggingChannel>,
    name: String,
}

impl Logger {
    pub fn new(logger_receiver: Receiver<LoggingChannel>, name: String) -> Logger {
        Logger {
            receiver: logger_receiver,
            name,
        }
    }

    pub fn run(&self) {
        for notification in &self.receiver {
            match notification.log_type {
                LogType::Error => self.error(&notification.message),
                LogType::Log => self.log(&notification.message),
                LogType::Warn => self.warn(&notification.message),
                LogType::Debug => self.debug(&notification.message),
            }
        }
    }

    fn log(&self, msg: &str) {
        self.print(msg, "LOG  ".green());
    }

    fn error(&self, msg: &str) {
        self.print(msg, "ERROR".red());
    }

    fn warn(&self, msg: &str) {
        self.print(msg, "WARN ".yellow());
    }

    fn debug(&self, msg: &str) {
        self.print(msg, "DEBUG".white());
    }

    fn print(&self, msg: &str, log_type: ColoredString) {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S");
        println!("{} [Rik] [{}] - {} - {}", log_type, self.name, now, msg);
    }
}

pub struct LoggingChannel {
    pub message: String,
    pub log_type: LogType,
}
