use chrono;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio_util::sync::CancellationToken;

mod cli;
mod data;
mod log_monitor;
mod read_line;
mod serial_monitor;
mod writer;

use log_monitor::AsyncLogMonitor;

/// Asynchronously gets single key from the user.
async fn get_key(
    stdin: &mut termion::input::Keys<termion::AsyncReader>,
) -> Result<termion::event::Key, Error> {
    loop {
        match stdin.next() {
            Some(c) => {
                return c;
            }
            None => {
                // wait for the next key
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DrFish is a fish doctor! 🐟");

    let port_configuration = match cli::get_port_configuration() {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting to read from serial ports. Press CTRL-C to exit.");

    let mut handles = Vec::new();
    let cancel_signal = CancellationToken::new();
    let (sender, mut receiver) =
        tokio::sync::mpsc::unbounded_channel::<log_monitor::MonitorMessage>();

    let mut writer = writer::Writer::new();

    for port in &port_configuration {
        let cancel_signal_clone = cancel_signal.clone();
        let sender_clone = sender.clone();
        let mut port_monitor = serial_monitor::SerialLogMonitor::new(port.clone())?;
        let write_proxy = port_monitor.get_write_proxy();
        writer.add_write_proxy(port.path.clone(), write_proxy);

        let handle = tokio::spawn(async move {
            port_monitor
                .monitor(cancel_signal_clone, sender_clone)
                .await;
        });
        handles.push(handle);
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file_name = format!("log_{}.txt", timestamp);
    let mut log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&log_file_name)?;

    let mut stdout = std::io::stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();

    let mut last_msg_was_unsolicited = false;

    loop {
        tokio::select! {
            msg = receiver.recv() => {
                if let Some(msg) = msg {
                    match msg {
                        log_monitor::MonitorMessage::UnsolictedMessage(msg) => {
                            print!("{}", msg);
                            stdout.flush()?;
                            last_msg_was_unsolicited = true;
                        }
                        log_monitor::MonitorMessage::Log(msg) => {
                            if last_msg_was_unsolicited {
                                print!("\r\n");
                            }
                            let log_msg = format!(">> [{}] | {}: {}\r\n", msg.timestamp, msg.source_name, msg.message);
                            print!("{}", &log_msg);
                            write!(log_file, "{}", &log_msg)?;
                            stdout.flush()?;
                            last_msg_was_unsolicited = false;
                        }

                    }
                }
            }

            _ = tokio::signal::ctrl_c() => {
                print!("Captured Ctrl+c, bye fisherman!\r\n");
                break;
            }

            key = get_key(&mut stdin) => {
                match key {
                    Ok(termion::event::Key::Ctrl('x')) => {
                        print!("Exiting...\r\n");
                        cancel_signal.cancel();
                        break;
                    }
                    Ok(key) => {
                        match key {
                            termion::event::Key::Char(c) => {
                                // FIXME: this is a hack to send CRLF to the serial port
                                if c == '\n' {
                                    writer.write_key(termion::event::Key::Char('\r'));
                                    continue;
                                }

                                writer.write_key(key);
                            }
                            termion::event::Key::Ctrl('z') => {
                                let new_writer_name = writer.switch_to_next_writer();
                                match new_writer_name {
                                    Some(_) => {
                                        print!("Switching to {}\r\n", new_writer_name.unwrap());
                                    }
                                    None => {
                                        print!("No other writer available\r\n");
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        print!("Error: {}\r\n", e);
                    }
                }
            }
        }
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
