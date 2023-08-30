use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::Error;

use self::{verack::VerackMessageBuilder, version::VersionMessageBuilder};

pub mod verack;
pub mod version;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C)]
pub struct MessageHeader {
    magin_network_nr: [u8; 4],
    pub command: [u8; 12],
    pub payload_len: u32,
    checksum: u32,
}

impl TryFrom<&[u8]> for MessageHeader {
    type Error = Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        Ok(bincode::deserialize(&buf[0..24])?)
    }
}

pub enum Message {
    Version(VersionMessageBuilder),
    Verack(VerackMessageBuilder),
}

impl ToNetworkMessage for Message {
    fn to_network_message(self) -> Result<Vec<u8>, Error> {
        match self {
            Message::Version(version_message) => {
                let btc_message: SerializedBitcoinMessage = version_message.try_into()?;
                Ok(btc_message.to_network_message()?)
            }
            Message::Verack(verack_message) => {
                let btc_message: SerializedBitcoinMessage = verack_message.try_into()?;
                Ok(btc_message.to_network_message()?)
            }
        }
    }
}

#[derive(Serialize)]
#[repr(C)]
struct SerializedBitcoinMessage {
    header: Vec<u8>,
    message: Vec<u8>,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageMagicNumber {
    Main,
    Testnet,
    Signet,
    Regtest,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MessageCommand {
    Version,
    Verack,
}

impl TryFrom<[u8; 12]> for MessageCommand {
    type Error = Error;

    fn try_from(value: [u8; 12]) -> Result<Self, Self::Error> {
        match value {
            [0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                Ok(Self::Version)
            }
            [0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                Ok(Self::Verack)
            }
            _ => Err(Error::Unexpected(anyhow!("Unexpected message"))),
        }
    }
}

pub trait ToNetworkMessage {
    fn to_network_message(self) -> Result<Vec<u8>, Error>;
}

impl From<MessageMagicNumber> for [u8; 4] {
    fn from(value: MessageMagicNumber) -> Self {
        match value {
            MessageMagicNumber::Main => [0xF9, 0xBE, 0xB4, 0xD9],
            MessageMagicNumber::Testnet => [0x0B, 0x11, 0x09, 0x07],
            MessageMagicNumber::Signet => [0x0A, 0x03, 0xCF, 0x40],
            MessageMagicNumber::Regtest => [0xFA, 0xBF, 0xB5, 0xDA],
        }
    }
}

impl From<MessageCommand> for [u8; 12] {
    fn from(value: MessageCommand) -> Self {
        match value {
            MessageCommand::Version => [
                0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            MessageCommand::Verack => [
                0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
        }
    }
}
impl ToNetworkMessage for SerializedBitcoinMessage {
    fn to_network_message(self) -> Result<Vec<u8>, Error> {
        Ok([&self.header[..], &self.message[..]].concat())
    }
}

pub fn htons(u: u16) -> u16 {
    u.to_be()
}

pub fn htonl(u: u32) -> u32 {
    u.to_be()
}

pub fn calc_checksum(paylod: &[u8]) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(paylod);
    let result = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(result.as_slice());
    let result = hasher.finalize();

    htonl(
        ((result[0] as u32) << 24)
            + ((result[1] as u32) << 16)
            + ((result[2] as u32) << 8)
            + (result[3] as u32),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_deserialize_version() {
        let version_hex: Vec<u8> = vec![
            0xF9, 0xBE, 0xB4, 0xD9, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x11, 0xF1, 0x11, 0x87, 0x71, 0x11, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x59, 0x16, 0xED, 0x64, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x4F, 0x74, 0x94, 0x76, 0x20, 0x8D,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3B, 0x2E,
            0xB3, 0x5D, 0x8C, 0xE6, 0x17, 0x65, 0x0E, 0x65, 0x6D, 0x69, 0x6C, 0x2D, 0x68, 0x61,
            0x6E, 0x64, 0x73, 0x68, 0x61, 0x6B, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let message_header: MessageHeader = version_hex[0..24].try_into().unwrap();
        let command: MessageCommand = message_header.command.try_into().unwrap();
        assert_eq!(MessageCommand::Version, command);
    }

    #[test]
    fn test_deserialize_verack() {
        let verack_hex: Vec<u8> = vec![
            0xF9, 0xBE, 0xB4, 0xD9, 0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0xE2, 0x5D, 0xF6,
        ];
        let message_header: MessageHeader = verack_hex[0..24].try_into().unwrap();
        let command: MessageCommand = message_header.command.try_into().unwrap();
        assert_eq!(MessageCommand::Verack, command);
    }
}
