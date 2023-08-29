use super::{MessageCommand, MessageHeader, MessageMagicNumber, SerializedBitcoinMessage};

pub struct VerackMessageBuilder {
    pub magic_number: MessageMagicNumber,
    pub command: MessageCommand,
}

impl VerackMessageBuilder {
    pub fn new(magic_number: MessageMagicNumber) -> Self {
        Self {
            magic_number,
            command: MessageCommand::Verack,
        }
    }
}
impl From<VerackMessageBuilder> for SerializedBitcoinMessage {
    fn from(value: VerackMessageBuilder) -> Self {
        let header = MessageHeader {
            magin_network_nr: value.magic_number.into(),
            command: value.command.into(),
            payload_len: 0,
            checksum: 0xF65DE2E0, //Magic value taken from docs
        };
        Self {
            header: bincode::serialize(&header).unwrap(),
            message: Vec::new(),
        }
    }
}
