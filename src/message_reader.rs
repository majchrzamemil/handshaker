use std::io::Read;

use crate::messages::{MessageCommand, MessageHeader};

const BUFFER_SIZE: usize = 1000;

pub struct MessageReader {
    buffer: [u8; BUFFER_SIZE], //Just a big buffer
    reader: Box<dyn Read>,
}

impl MessageReader {
    const HEADER_SIZE: u64 = 24;
    pub fn new(reader: Box<dyn Read>) -> Self {
        Self {
            buffer: [0x0; 1000],
            reader,
        }
    }

    pub fn read_message(&mut self) -> Option<MessageCommand> {
        // Read header
        let mut take = self.reader.as_mut().take(MessageReader::HEADER_SIZE);
        let readed = take
            .read(&mut self.buffer[0..MessageReader::HEADER_SIZE as usize])
            .unwrap();
        if readed == 0 {
            return None;
        }
        let header: MessageHeader = self.buffer.as_ref().try_into().unwrap();
        let command: MessageCommand = match header.command.try_into() {
            Ok(command) => command,
            Err(_) => return None,
        };
        // Read the rest of the payload
        let mut data_to_read = header.payload_len as usize;
        loop {
            let bytes_to_read = if data_to_read > BUFFER_SIZE {
                BUFFER_SIZE
            } else {
                data_to_read
            };
            let mut take = self.reader.as_mut().take(bytes_to_read as u64);
            let readed = take.read(&mut self.buffer).unwrap();
            data_to_read = data_to_read - readed;
            if data_to_read <= 0 {
                break;
            }
        }
        Some(command)
    }
}