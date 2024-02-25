use crate::data;
use crate::log_monitor;
use crate::logging;
use crate::serial_monitor;
use crate::writer;

use log_monitor::AsyncLogMonitor;
use tokio_util::sync::CancellationToken;

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyUSB0";
const DEFAULT_BAUD_RATE: u32 = 115_200;

pub struct DrFishCli {
    pub port_configuration: Vec<data::SerialPortSettings>,
    pub logger: logging::Logger,

    writer: writer::Writer,
    sender: tokio::sync::mpsc::UnboundedSender<log_monitor::MonitorMessage>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<log_monitor::MonitorMessage>,
    cancel_signal: tokio_util::sync::CancellationToken,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

pub enum CliAction {
    Break,
}

impl DrFishCli {
    pub fn new() -> Result<DrFishCli, String> {
        let port_configuration = match get_port_configuration() {
            Ok(cfg) => cfg,
            Err(e) => {
                return Err(e);
            }
        };

        let logger = match logging::Logger::new() {
            Ok(l) => l,
            Err(e) => {
                return Err(e);
            }
        };

        let writer = writer::Writer::new();
        let (sender, receiver) =
            tokio::sync::mpsc::unbounded_channel::<log_monitor::MonitorMessage>();
        let cancel_signal = CancellationToken::new();
        let handles = Vec::new();

        Ok(DrFishCli {
            port_configuration: port_configuration,
            writer: writer,
            sender: sender,
            receiver: receiver,
            cancel_signal: cancel_signal,
            handles: handles,
            logger: logger,
        })
    }

    pub fn spawn_monitors(&mut self) {
        for port in &self.port_configuration {
            let cancel_signal_clone = self.cancel_signal.clone();
            let sender_clone = self.sender.clone();
            let mut port_monitor = serial_monitor::SerialLogMonitor::new(port.clone()).unwrap();
            let write_proxy = port_monitor.get_write_proxy();
            self.writer.add_write_proxy(port.path.clone(), write_proxy);

            let handle = tokio::spawn(async move {
                port_monitor
                    .monitor(cancel_signal_clone, sender_clone)
                    .await;
            });
            self.handles.push(handle);
        }
    }

    pub async fn stop_monitors(&mut self) {
        self.cancel_signal.cancel();
        for handle in &mut self.handles {
            handle.await.unwrap();
        }
    }

    pub async fn recieve_monitor_message(&mut self) -> Option<log_monitor::MonitorMessage> {
        self.receiver.recv().await
    }

    pub async fn handle_key_press(&mut self, key: termion::event::Key) -> Option<CliAction> {
        match key {
            termion::event::Key::Ctrl('x') => {
                print!("Exiting...\r\n");
                self.stop_monitors().await;
                return Some(CliAction::Break);
            }

            termion::event::Key::Ctrl('z') => {
                let new_writer_name = self.writer.switch_to_next_writer();
                match new_writer_name {
                    Some(_) => {
                        print!("Switching to {}\r\n", new_writer_name.unwrap());
                    }
                    None => {
                        print!("No other writer available\r\n");
                    }
                }
            }

            termion::event::Key::Ctrl(_) => {
                self.writer.write_key(key);
            }

            termion::event::Key::Char(c) => {
                // FIXME: this is a hack to send CRLF to the serial port
                if c == '\n' {
                    self.writer.write_key(termion::event::Key::Char('\r'));
                    return None;
                }

                self.writer.write_key(key);
            }

            _ => {}
        }

        None
    }
}

fn parse_port_arg(arg: &str) -> Result<data::SerialPortSettings, String> {
    if arg.is_empty() {
        return Err("Empty port argument".to_string());
    }

    let parts: Vec<&str> = arg.split(':').collect();
    if parts.len() == 1 {
        return Ok(data::SerialPortSettings {
            path: parts[0].to_string(),
            baud_rate: DEFAULT_BAUD_RATE,
        });
    } else if parts.len() != 2 {
        return Err(format!("Invalid port argument: {}", arg));
    }

    let port_path = parts[0].to_string();
    let baud_rate = match parts[1].parse::<u32>() {
        Ok(b) => b,
        Err(_) => {
            return Err(format!("Invalid baud rate: {}", parts[1]));
        }
    };

    Ok(data::SerialPortSettings {
        path: port_path,
        baud_rate: baud_rate,
    })
}

/// returns a collection of serial port settings from CLI arguments
/// or a default value if no arguments are provided
///
/// serial ports can be passed as arguments in the form of:
/// PORT_PATH,BAUD_RATE (e.g. /dev/ttyUSB0:115200)
pub fn get_port_configuration() -> Result<Vec<data::SerialPortSettings>, String> {
    let mut settings = Vec::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        settings.push(data::SerialPortSettings {
            path: DEFAULT_SERIAL_PORT.to_string(),
            baud_rate: DEFAULT_BAUD_RATE,
        });
    } else {
        for arg in &args[1..] {
            let port_settings = parse_port_arg(arg);
            match port_settings {
                Ok(s) => settings.push(s),
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_arg() {
        let port_arg = "path:115200";
        let result = parse_port_arg(port_arg);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.path, "path");
        assert_eq!(settings.baud_rate, 115200);
    }

    #[test]
    fn test_parse_port_arg_invalid() {
        let port_arg = "path:invalid";
        let result = parse_port_arg(port_arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_port_empty() {
        let port_arg = "";
        let result = parse_port_arg(port_arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_port_default_baudrate() {
        let port_arg = "/dev/ttyUSB0";
        let result = parse_port_arg(port_arg);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.path, "/dev/ttyUSB0");
        assert_eq!(settings.baud_rate, 115200);
    }

    #[test]
    fn test_parse_port_invalid_fields() {
        let port_arg = "path:115200:extra";
        let result = parse_port_arg(port_arg);
        assert!(result.is_err());
    }
}
