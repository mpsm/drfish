use tokio::io::AsyncReadExt;
use tokio::signal;

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyUSB0";
const DEFAULT_BAUD_RATE: u32 = 115200;

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

    let port_builder = tokio_serial::new(port_name, DEFAULT_BAUD_RATE);
    let mut serial_stream = match tokio_serial::SerialStream::open(&port_builder) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to open serial port: {}", e);
            return Ok(());
        }
    };

    let mut buffer = vec![0; 1024];

    println!("Starting to read from serial port. Press CTRL-C to exit.");

    loop {
        tokio::select! {
            _ = serial_stream.read(&mut buffer) => {
                let s = String::from_utf8_lossy(&buffer);
                println!(">> Received: {}", s);
            }

            _ = signal::ctrl_c() => {
                println!(">> Received CTRL-C. Exiting...");
                return Ok(());
            }
        }
    }
}
