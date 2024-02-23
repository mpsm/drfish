use super::data::SerialPortSettings;
use super::log_monitor::{AsyncLogMonitor, Log};
use super::read_line;

use chrono;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_serial::SerialStream;
use tokio_util::sync::CancellationToken;

const DEFAULT_BUFFER_SIZE: usize = 4096;

pub struct SerialLogMonitor {
    port_settings: SerialPortSettings,
    serial_stream: SerialStream,
    write_receiver: UnboundedReceiver<u8>,
    write_sender: UnboundedSender<u8>,
}

pub struct SerialLogMonitorWriteProxy {
    write_sender: UnboundedSender<u8>,
}

impl SerialLogMonitorWriteProxy {
    pub fn new(write_sender: UnboundedSender<u8>) -> SerialLogMonitorWriteProxy {
        SerialLogMonitorWriteProxy {
            write_sender: write_sender,
        }
    }

    pub fn send(&self, data: u8) {
        self.write_sender.send(data).unwrap();
    }
}

impl SerialLogMonitor {
    pub fn new(port_settings: SerialPortSettings) -> Result<SerialLogMonitor, String> {
        let port_builder = tokio_serial::new(port_settings.path.clone(), port_settings.baud_rate);
        let serial_stream = match tokio_serial::SerialStream::open(&port_builder) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to open serial port: {}", e)),
        };

        let (write_sender, write_receiver) = tokio::sync::mpsc::unbounded_channel::<u8>();

        Ok(SerialLogMonitor {
            port_settings: port_settings,
            serial_stream: serial_stream,
            write_receiver: write_receiver,
            write_sender: write_sender,
        })
    }

    pub fn get_write_proxy(&self) -> SerialLogMonitorWriteProxy {
        SerialLogMonitorWriteProxy::new(self.write_sender.clone())
    }
}

impl AsyncLogMonitor for SerialLogMonitor {
    fn get_common_name(&self) -> String {
        self.port_settings.path.clone()
    }

    async fn monitor(
        &mut self,
        cancel_token: CancellationToken,
        sender_queue: UnboundedSender<Log>,
    ) {
        let mut recv_buffer = vec![0; DEFAULT_BUFFER_SIZE];
        let mut process_buffer = vec![];

        print!(
            "Starting {} port monitor @ {}\r\n",
            self.port_settings.path, self.port_settings.baud_rate
        );

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return;
                }

                write_result = self.write_receiver.recv() => {
                    match write_result {
                        Some(data) => {
                            match self.serial_stream.write(&[data]).await {
                                Ok(_) => {}
                                Err(_) => {
                                    continue;
                                }
                            }
                        }
                        None => {
                            continue;
                        }
                    }
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
                                let stripped_line = line.trim().to_string();
                                let message = Log {
                                    source_name: self.get_common_name(),
                                    message: stripped_line,
                                    timestamp: chrono::Local::now(),
                                };
                                sender_queue.send(message).unwrap();
                            }
                        }
                        // FIXME: handle the error
                        Err(_) => {
                            return;
                        }
                    }
                }
            }
        }
    }
}
