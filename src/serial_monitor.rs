use super::log_monitor::{AsyncLogMonitor, Log};
use super::read_line;
use chrono;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio_serial::SerialStream;
use tokio_util::sync::CancellationToken;

const DEFAULT_BAUD_RATE: u32 = 115_200;
const DEFAULT_BUFFER_SIZE: usize = 4096;

pub struct SerialLogMonitor {
    port_path: String,
    serial_stream: SerialStream,
}

impl SerialLogMonitor {
    pub fn new(port_path: &str) -> Result<SerialLogMonitor, String> {
        let port_builder = tokio_serial::new(port_path, DEFAULT_BAUD_RATE);
        let serial_stream = match tokio_serial::SerialStream::open(&port_builder) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to open serial port: {}", e)),
        };

        Ok(SerialLogMonitor {
            port_path: port_path.to_string(),
            serial_stream: serial_stream,
        })
    }
}

impl AsyncLogMonitor for SerialLogMonitor {
    fn get_common_name(&self) -> String {
        self.port_path.clone()
    }

    async fn monitor(
        &mut self,
        cancel_token: CancellationToken,
        sender_queue: UnboundedSender<Log>,
    ) {
        let mut recv_buffer = vec![0; DEFAULT_BUFFER_SIZE];
        let mut process_buffer = vec![];

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    println!("Closing {} port monitor", self.port_path);
                    return;
                }

                read_result = self.serial_stream.read(&mut recv_buffer) => {
                    match read_result {
                        Ok(n) => {
                            if n == 0 {
                                continue;
                            }

                            // concatenate the new data to the process buffer
                            process_buffer.extend_from_slice(&recv_buffer[0..n]);

                            while let Some(line) = read_line::read_line_from_buffer(&mut process_buffer) {
                                let message = Log {
                                    source_name: self.get_common_name(),
                                    message: line,
                                    timestamp: chrono::Local::now(),
                                };
                                sender_queue.send(message).unwrap();
                            }
                        }
                        Err(e) => {
                            println!("Failed to read from serial port: {}", e);
                            return;
                        }
                    }
                }
            }
        }
    }
}
