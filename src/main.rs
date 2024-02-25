use std::io::Error;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod cli;
mod data;
mod log_monitor;
mod logging;
mod read_line;
mod serial_monitor;
mod writer;

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

    let mut dr_fish = match cli::DrFishCli::new() {
        Ok(cli) => cli,
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting to read from serial ports. Press CTRL-X to exit and CTRL-Z to switch input.");

    dr_fish.spawn_monitors();

    let mut stdout = std::io::stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();

    loop {
        tokio::select! {
            msg = dr_fish.recieve_monitor_message() => {
                if let Some(msg) = msg {
                    dr_fish.logger.log_monitor_message_to_file(&msg);
                    dr_fish.logger.log_monitor_message_to_stdout(&msg, &mut stdout);
                }
            }

            _ = tokio::signal::ctrl_c() => {
                print!("Captured Ctrl+c, bye fisherman!\r\n");
                break;
            }

            key = get_key(&mut stdin) => {
                if key.is_err() {
                    print!("Error on key press: {}\r\n", key.err().unwrap());
                    continue;
                }
                if let Some(cli::CliAction::Break) = dr_fish.handle_key_press(key.unwrap()).await {
                    break;
                }
            }
        }
    }

    Ok(())
}
