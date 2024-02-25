use crate::log_monitor;

use chrono;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub struct Logger {
    log_file: File,
    last_msg_was_unsolicited: bool,
}

impl Logger {
    pub fn new() -> Result<Logger, String> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_file_name = format!("log_{}.txt", timestamp);
        let log_file = match OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&log_file_name)
        {
            Ok(file) => file,
            Err(e) => {
                return Err(format!("Failed to open log file: {}", e));
            }
        };

        Ok(Logger {
            log_file: log_file,
            last_msg_was_unsolicited: false,
        })
    }

    pub fn log_monitor_message_to_file(&mut self, msg: &log_monitor::MonitorMessage) {
        match msg {
            log_monitor::MonitorMessage::UnsolictedMessage(msg) => {
                write!(self.log_file, ">>> {}", msg).unwrap();
            }
            log_monitor::MonitorMessage::Log(msg) => {
                let log_msg = format!(
                    ">> [{}] | {}: {}\r\n",
                    msg.timestamp, msg.source_name, msg.message
                );
                write!(self.log_file, "{}", &log_msg).unwrap();
            }
        }
    }

    pub fn log_monitor_message_to_stdout(
        &mut self,
        msg: &log_monitor::MonitorMessage,
        stdout: &mut std::io::Stdout,
    ) {
        match msg {
            log_monitor::MonitorMessage::UnsolictedMessage(msg) => {
                self.last_msg_was_unsolicited = true;
                print!("{}", msg);
                stdout.flush().unwrap();
            }
            log_monitor::MonitorMessage::Log(msg) => {
                if self.last_msg_was_unsolicited {
                    print!("\r\n");
                }
                self.last_msg_was_unsolicited = false;
                let log_msg = format!(
                    ">> [{}] | {}: {}\r\n",
                    msg.timestamp, msg.source_name, msg.message
                );
                print!("{}", &log_msg);
                stdout.flush().unwrap();
            }
        }
    }
}
