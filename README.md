# Doctor Fish [![Rust CI](https://github.com/mpsm/drfish/actions/workflows/rust.yml/badge.svg)](https://github.com/mpsm/drfish/actions/workflows/rust.yml)

Doctor Fish is a serial port monitor that will also serve as a log analyzer in the future.


## Installation

To build and install the application, you first need to [set up the Rust compiler](https://www.rust-lang.org/tools/install).

`drfish` relies on the `serialport` crate, which requires the following dependencies on Ubuntu:
```bash
sudo apt install pkg-config libudev-dev
```

For other systems, please refer to the serialport crate documentation.

To install `drfish`, clone this repository and run:
```bash
cargo install --path .
```

## Usage

To use drfish, run the command with one or more ports listed as command line arguments:
```bash
drfish /dev/ttyUSB0 /dev/ttyUSB2
```
You can specify the baud rate after the : character, like so:
```bash
drfish /dev/ttyUSB0:115200 /dev/ttyUSB1:9600
```
The default baud rate is `115200`. If no arguments are provided, the application will monitor `/dev/ttyUSB0` at `115200`.

Every session is logged. Please check your current working directory for the log file.

### Key bindings

Upon launching, user input is sent to the first serial port passed as an argument. Only regular ASCII characters and Ctrl+key combinations are passed, with the exception of the following:

| Key Binding | Action |
| ----------- | ------ |
| Ctrl + Z    | Switch input to the next console (if more than one is opened) |
| Ctrl + X    | Exit the application |
