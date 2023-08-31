use crate::error::Error;

use super::{htonl, MessageCommand, MessageHeader, MessageMagicNumber, SerializedBitcoinMessage};

/// Represents a builder for creating a Verack message.
pub struct VerackMessageBuilder {
    /// The magic number for the Bitcoin network.
    pub magic_number: MessageMagicNumber,
    /// The message command (always MessageCommand::Verack).
    pub command: MessageCommand,
}

impl VerackMessageBuilder {
    /// Creates a new instance of `VerackMessageBuilder`.
    ///
    /// # Arguments
    ///
    /// * `magic_number` - The magic number for the Bitcoin network.
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
            checksum: htonl(0x5df6e0e2), //Magic value taken from docs
        };
        Ok(Self {
            header: bincode::serialize(&header)?,
            message: Vec::new(),
        })
    }
}
