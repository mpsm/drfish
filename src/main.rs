use serialport;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_SERIAL_PORT_BAUD_RATE: u32 = 115200;
const DEFAULT_BUFFER_SIZE: usize = 100;

fn main() {
    println!("DrFish is a fish doctor! 🐟👨‍⚕️");

    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Usage: drfish <serial port>");
        return;
    }

    let port_name = &args[1];
    let mut port = serialport::new(port_name, DEFAULT_SERIAL_PORT_BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open serial port");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut buffer: Vec<u8> = vec![0; DEFAULT_BUFFER_SIZE];
    while running.load(Ordering::SeqCst) {
        match port.read(&mut buffer) {
            Ok(n) => {
                for b in buffer[..n].iter() {
                    print!("{}", *b as char);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
