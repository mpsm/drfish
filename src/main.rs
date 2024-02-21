use chrono;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod cli;
mod data;
mod log_monitor;
mod read_line;
mod serial_monitor;

use log_monitor::AsyncLogMonitor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DrFish is a fish doctor! 🐟");

    let port_configuration = cli::get_port_configuration();
    match port_configuration {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }

    println!("Starting to read from serial ports. Press CTRL-C to exit.");

    let mut handles = Vec::new();
    let cancel_signal = CancellationToken::new();
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<log_monitor::Log>();

    for port in port_configuration.unwrap() {
        let cancel_signal_clone = cancel_signal.clone();
        let sender_clone = sender.clone();
        let mut port_monitor = serial_monitor::SerialLogMonitor::new(port)?;
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

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!(">> Received CTRL-C. Exiting...");

                cancel_signal.cancel();
                break;
            }

            msg = receiver.recv() => {
                if let Some(msg) = msg {
                    let log_msg = format!(">> [{}] | {}: {}", msg.timestamp, msg.source_name, msg.message);
                    print!("{}", &log_msg);
                    write!(log_file, "{}", &log_msg)?;
                }
            }
        }
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
