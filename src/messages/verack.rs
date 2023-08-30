use crate::error::Error;

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
impl TryFrom<VerackMessageBuilder> for SerializedBitcoinMessage {
    type Error = Error;

    fn try_from(value: VerackMessageBuilder) -> Result<Self, Self::Error> {
        let header = MessageHeader {
            magin_network_nr: value.magic_number.into(),
            command: value.command.into(),
            payload_len: 0,
            checksum: 0xF65DE2E0, //Magic value taken from docs
        };
        Ok(Self {
            header: bincode::serialize(&header)?,
            message: Vec::new(),
        })
    }
}
