use super::data;

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyUSB0";
const DEFAULT_BAUD_RATE: u32 = 115_200;

fn parse_port_arg(arg: &str) -> Result<data::SerialPortSettings, String> {
    if arg.is_empty() {
        return Err("Empty port argument".to_string());
    }

    let parts: Vec<&str> = arg.split(':').collect();
    if parts.len() == 1 {
        return Ok(data::SerialPortSettings {
            path: parts[0].to_string(),
            baud_rate: DEFAULT_BAUD_RATE,
        });
    } else if parts.len() != 2 {
        return Err(format!("Invalid port argument: {}", arg));
    }

    let port_path = parts[0].to_string();
    let baud_rate = match parts[1].parse::<u32>() {
        Ok(b) => b,
        Err(_) => {
            return Err(format!("Invalid baud rate: {}", parts[1]));
        }
    };

    Ok(data::SerialPortSettings {
        path: port_path,
        baud_rate: baud_rate,
    })
}

/// returns a collection of serial port settings from CLI arguments
/// or a default value if no arguments are provided
///
/// serial ports can be passed as arguments in the form of:
/// PORT_PATH,BAUD_RATE (e.g. /dev/ttyUSB0:115200)
pub fn get_port_configuration() -> Result<Vec<data::SerialPortSettings>, String> {
    let mut settings = Vec::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        settings.push(data::SerialPortSettings {
            path: DEFAULT_SERIAL_PORT.to_string(),
            baud_rate: DEFAULT_BAUD_RATE,
        });
    } else {
        for arg in &args[1..] {
            let port_settings = parse_port_arg(arg);
            match port_settings {
                Ok(s) => settings.push(s),
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_arg() {
        let port_arg = "path:115200";
        let result = parse_port_arg(port_arg);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.path, "path");
        assert_eq!(settings.baud_rate, 115200);
    }

    #[test]
    fn test_parse_port_arg_invalid() {
        let port_arg = "path:invalid";
        let result = parse_port_arg(port_arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_port_empty() {
        let port_arg = "";
        let result = parse_port_arg(port_arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_port_default_baudrate() {
        let port_arg = "/dev/ttyUSB0";
        let result = parse_port_arg(port_arg);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.path, "/dev/ttyUSB0");
        assert_eq!(settings.baud_rate, 115200);
    }

    #[test]
    fn test_parse_port_invalid_fields() {
        let port_arg = "path:115200:extra";
        let result = parse_port_arg(port_arg);
        assert!(result.is_err());
    }
}
