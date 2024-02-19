use tokio::io::AsyncReadExt;
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod read_line;

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyUSB0";
const DEFAULT_BAUD_RATE: u32 = 115200;
const DEFAULT_BUFFER_SIZE: usize = 4096;

async fn monitor_port(port_name: &String, cancel_token: CancellationToken) {
    let port_builder = tokio_serial::new(port_name, DEFAULT_BAUD_RATE);
    let mut serial_stream = match tokio_serial::SerialStream::open(&port_builder) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to open serial port: {}", e);
            return;
        }
    };

    let mut recv_buffer = vec![0; DEFAULT_BUFFER_SIZE];
    let mut process_buffer = vec![];

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                println!("Closing {} port monitor", port_name);
                return;
            }

            read_result = serial_stream.read(&mut recv_buffer) => {
                match read_result {
                    Ok(n) => {
                        if n == 0 {
                            continue;
                        }

                        // concatenate the new data to the process buffer
                        process_buffer.extend_from_slice(&recv_buffer[0..n]);

                        while let Some(line) = read_line::read_line_from_buffer(&mut process_buffer) {
                            print!("{}\t| {}", port_name, line);
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DrFish is a fish doctor! 🐟");

    let mut port_names = std::env::args().skip(1).collect::<Vec<_>>();
    if port_names.is_empty() {
        println!(
            "No serial port name provided. Using default: {}",
            DEFAULT_SERIAL_PORT
        );
        port_names.push(DEFAULT_SERIAL_PORT.to_string());
    }

    println!("Starting to read from serial ports. Press CTRL-C to exit.");

    let mut handles = Vec::new();
    let cancel_signal = CancellationToken::new();

    for port_name in port_names {
        let port_name_clone = port_name.clone();
        let cancel_signal_clone = cancel_signal.clone();
        let handle =
            tokio::spawn(async move { monitor_port(&port_name_clone, cancel_signal_clone).await });
        handles.push(handle);
    }

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!(">> Received CTRL-C. Exiting...");

                cancel_signal.cancel();
                break;
            }
        }
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
