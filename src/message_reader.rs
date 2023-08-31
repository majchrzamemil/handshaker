use std::io::Read;

use crate::{
    error::Error,
    messages::message::{MessageCommand, MessageHeader},
};

/// The size of the buffer used for reading from the stream.
const BUFFER_SIZE: usize = 1024; //1k just because

/// Represents a reader for Bitcoin messages.
pub struct MessageReader {
    buffer: [u8; BUFFER_SIZE],
    reader: Box<dyn Read>,
}

impl MessageReader {
    /// The size of a message header.
    const HEADER_SIZE: u64 = 24;

    /// Creates a new instance of `MessageReader` with the given reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader implementing the `Read` trait.
    pub fn new(reader: Box<dyn Read>) -> Self {
        Self {
            buffer: [0x0; BUFFER_SIZE],
            reader,
        }
    }
    /// Reads a Bitcoin message from the underlying stream.
    ///
    /// Returns the parsed `MessageCommand` if successful. If the read was successful
    /// but no more data is available (end of stream), or message is unrecognized `Ok(None)` is returned.
    /// If an error occurs during reading or parsing, an `Error` is returned.
    pub fn read_message(&mut self) -> Result<Option<MessageCommand>, Error> {
        // Read header
        let mut take = self.reader.as_mut().take(MessageReader::HEADER_SIZE);
        let readed = take.read(&mut self.buffer[0..MessageReader::HEADER_SIZE as usize])?;

        if readed == 0 {
            return Ok(None);
        }

        // Parse header
        let header: MessageHeader = self.buffer.as_ref().try_into()?;
        let command: MessageCommand = match header.command.try_into() {
            Ok(command) => command,
            Err(e) => {
                eprintln!("{e}");
                return Ok(None);
            }
        };

        // Read the rest of the payload
        let mut data_to_read = header.payload_len as usize;
        // Limiting number of iteration so we won't end up in infinite loop
        let mut it = 0;
        loop {
            let bytes_to_read = if data_to_read > BUFFER_SIZE {
                BUFFER_SIZE
            } else {
                data_to_read
            };
            let mut take = self.reader.as_mut().take(bytes_to_read as u64);
            let readed = take.read(&mut self.buffer)?;
            data_to_read -= readed;
            it += 1;
            if data_to_read == 0 || it > 10 {
                break;
            }
        }
        Ok(Some(command))
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;
    #[test]
    fn test_big_payload() {
        let mut big_message: Vec<u8> = vec![
            0xF9, 0xBE, 0xB4, 0xD9, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x64, 0x0A, 0x00, 0x00, 0x11, 0xF1, 0x11, 0x87, 0x71, 0x11, 0x01, 0x00,
        ];
        let mut dummy_data: Vec<u8> = vec![0; 3000];
        big_message.append(&mut dummy_data);
        let cursor = Cursor::new(big_message);
        let mut reader = MessageReader::new(Box::new(cursor));
        reader.read_message().unwrap();
    }
}
