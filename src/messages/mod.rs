use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

impl TryFrom<Vec<u8>> for MessageHeader {
    type Error = ();

    fn try_from(_value: Vec<u8>) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

#[derive(Serialize)]
#[repr(C)]
pub struct SerializedBitcoinMessage {
    pub header: Vec<u8>,
    pub message: Vec<u8>,
}

pub enum MessageMagicNumber {
    Main,
}

#[derive(Debug)]
pub enum MessageCommand {
    Version,
    Verack,
}

impl TryFrom<[u8; 12]> for MessageCommand {
    type Error = ();

    fn try_from(value: [u8; 12]) -> Result<Self, Self::Error> {
        match value {
            [0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                Ok(Self::Version)
            }
            [0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                Ok(Self::Verack)
            }
            _ => Err(()), //TODO: proper error, unrecognized command
        }
    }
}

pub trait ToNetworkMessage {
    fn to_network_message(&self) -> Vec<u8>;
}

impl From<MessageMagicNumber> for [u8; 4] {
    fn from(value: MessageMagicNumber) -> Self {
        match value {
            MessageMagicNumber::Main => [0xF9, 0xBE, 0xB4, 0xD9],
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
    fn to_network_message(&self) -> Vec<u8> {
        [&self.header[..], &self.message[..]].concat()
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
            + ((result[3] as u32) << 0),
    )
}