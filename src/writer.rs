use super::serial_monitor::SerialLogMonitorWriteProxy;
use indexmap;

pub struct Writer {
    write_proxies: indexmap::IndexMap<String, SerialLogMonitorWriteProxy>,
    current_writer_index: Option<u32>,
}

impl Writer {
    pub fn new() -> Self {
        Writer {
            write_proxies: indexmap::IndexMap::new(),
            current_writer_index: None,
        }
    }

    pub fn add_write_proxy(
        &mut self,
        common_name: String,
        write_proxy: SerialLogMonitorWriteProxy,
    ) {
        if self.current_writer_index.is_none() {
            self.current_writer_index = Some(0);
        }
        self.write_proxies.insert(common_name, write_proxy);
    }

    pub fn get_current_writer_name(&self) -> Option<String> {
        match self.current_writer_index {
            Some(index) => {
                let (name, _) = self.write_proxies.get_index(index as usize).unwrap();
                Some(name.clone())
            }
            None => None,
        }
    }

    pub fn switch_to_next_writer(&mut self) -> Option<String> {
        if self.write_proxies.len() <= 1 {
            return None;
        }

        self.current_writer_index = match self.current_writer_index {
            Some(index) => {
                if index + 1 < self.write_proxies.len() as u32 {
                    Some(index + 1)
                } else {
                    Some(0)
                }
            }
            None => None,
        };

        return self.get_current_writer_name();
    }

    pub fn write_key(&self, key: termion::event::Key) {
        let current_writer = match self.get_current_writer() {
            Some(writer) => writer,
            None => return,
        };

        match key {
            termion::event::Key::Char(c) => {
                current_writer.send(c as u8);
            }
            termion::event::Key::Ctrl(c) => {
                let ascii = (c.to_uppercase().next().unwrap() as u8) - 64;
                current_writer.send(ascii);
            }
            _ => {}
        }
    }

    fn get_current_writer(&self) -> Option<&SerialLogMonitorWriteProxy> {
        match self.current_writer_index {
            Some(index) => {
                let (_, write_proxy) = self.write_proxies.get_index(index as usize).unwrap();
                Some(write_proxy)
            }
            None => None,
        }
    }
}
