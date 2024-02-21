# Doctor Fish

Doctor Fish - a serial port monitor and log analyzer in the future.

## Installation

To build and install the app, first [setup rust compiler](https://www.rust-lang.org/tools/install).

`drfish` uses `serialport` crate under the hood that require following dependencies on Ubuntu

For other systems please refer to [`serialport`](https://github.com/serialport/serialport-rs) crate documentation.

To install `drfish` clone this repo and run:
```bash
cargo install --path .
```

## Usage

Run `drfish` with one or multiple ports in command line arguments list:
```bash
drfish /dev/ttyUSB0 /dev/ttyUSB2
```
You can specify baudrate after `:` character, e.g.:
```bash
drfish /dev/ttyUSB0:115200 /dev/ttyUSB1:9600
```
Default baudrate is `115200`. If no arguments are provided, the app will monitor `/dev/ttyUSB0` at `115200`.

Every session is logged, please check your current working directory for a log file.