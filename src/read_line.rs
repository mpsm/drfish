pub fn read_line_from_buffer(buffer: &mut Vec<u8>) -> Option<String> {
    if let Some(i) = buffer.iter().position(|&x| x == b'\n') {
        let line = String::from_utf8_lossy(&buffer[..=i]).to_string();
        buffer.drain(..=i);
        Some(line)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_line_from_buffer_multiple_lines() {
        let mut buffer = "hello\nworld\n".as_bytes().to_vec();
        let line = read_line_from_buffer(&mut buffer).unwrap();
        assert_eq!(line, "hello\n");
        assert_eq!(buffer, "world\n".as_bytes().to_vec());
    }

    #[test]
    fn test_read_line_from_buffer_no_newline() {
        let mut buffer = "hello".as_bytes().to_vec();
        let result = read_line_from_buffer(&mut buffer);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_line_from_buffer_split_line() {
        let mut buffer1 = "hello\nworld".as_bytes().to_vec();
        let line1 = read_line_from_buffer(&mut buffer1).unwrap();
        assert_eq!(line1, "hello\n");
        assert_eq!(buffer1, "world".as_bytes().to_vec());

        let mut buffer2 = " part2\nend\n".as_bytes().to_vec();
        buffer2.splice(0..0, buffer1); // prepend remaining data from buffer1 to buffer2
        let line2 = read_line_from_buffer(&mut buffer2).unwrap();
        assert_eq!(line2, "world part2\n");
        assert_eq!(buffer2, "end\n".as_bytes().to_vec());

        let line3 = read_line_from_buffer(&mut buffer2).unwrap();
        assert_eq!(line3, "end\n");
        assert!(buffer2.is_empty());
    }
}
