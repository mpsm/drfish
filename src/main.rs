use tokio::io::AsyncReadExt;
use tokio::signal;

mod read_line;

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyUSB0";
const DEFAULT_BAUD_RATE: u32 = 115200;
const DEFAULT_BUFFER_SIZE: usize = 4096;

async fn monitor_port(port_name: &String) {
    let port_builder = tokio_serial::new(port_name, DEFAULT_BAUD_RATE);
    let mut serial_stream = match tokio_serial::SerialStream::open(&port_builder) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to open serial port: {}", e);
            return;
        }
    };

    let mut buffer = vec![0; DEFAULT_BUFFER_SIZE];
    let mut leftover_buffer = vec![0; DEFAULT_BUFFER_SIZE];

    loop {
        let read_result = serial_stream.read(&mut buffer).await;
        match read_result {
            Ok(n) => {
                if n == 0 {
                    continue;
                }

                if leftover_buffer.len() > 0 {
                    buffer.splice(0..0, leftover_buffer);
                }

                while let Ok(line) = read_line::read_line_from_buffer(&mut buffer) {
                    println!("Received: {}", line);
                }

                leftover_buffer = buffer.clone();
            }
            Err(e) => {
                println!("Failed to read from serial port: {}", e);
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DrFish is a fish doctor! 🐟");

    let port_name = std::env::args().nth(1).unwrap_or_else(|| {
        println!(
            "No serial port name provided. Using default: {}",
            DEFAULT_SERIAL_PORT
        );
        DEFAULT_SERIAL_PORT.to_string()
    });

    println!("Starting to read from serial port. Press CTRL-C to exit.");

    tokio::spawn(async move { monitor_port(&port_name).await });

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!(">> Received CTRL-C. Exiting...");
                return Ok(());
            }
        }
    }
}
