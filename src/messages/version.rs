use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::{
    calc_checksum, htons, MessageCommand, MessageHeader, MessageMagicNumber,
    SerializedBitcoinMessage, ToNetworkMessage,
};

pub struct VersionMessageBuilder {
    pub magic_number: MessageMagicNumber,
    pub command: MessageCommand,
    pub version: i32,
    pub timestamp: i64,
    pub addr_recv: SocketAddr,
    pub addr_from: SocketAddr,
    pub nonce: u64,
}

impl VersionMessageBuilder {
    const UA: [u8; 14] = [
        // UA is emil-handshake
        0x65, 0x6D, 0x69, 0x6C, 0x2D, 0x68, 0x61, 0x6E, 0x64, 0x73, 0x68, 0x61, 0x6B, 0x65,
    ];
    pub fn new(
        magic_number: MessageMagicNumber,
        addr_recv: SocketAddr,
        timestamp: i64,
        nonce: u64,
    ) -> Self {
        Self {
            magic_number,
            command: MessageCommand::Version,
            version: 70001,
            timestamp,
            addr_recv,
            addr_from: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            nonce,
        }
    }
}

impl TryFrom<VersionMessageBuilder> for SerializedBitcoinMessage {
    type Error = Error;

    fn try_from(value: VersionMessageBuilder) -> Result<Self, Self::Error> {
        let recv_add = value.addr_recv.into();
        let addr_from = value.addr_from.into();

        let message = VersionMessage {
            version: value.version,
            services: 0,
            timestamp: value.timestamp,
            recv_add,
            addr_from,
            nonce: value.nonce,
            ua_len: VersionMessageBuilder::UA.len() as u8,
            user_agent: VersionMessageBuilder::UA,
            start_height: 0,
            relay: false,
        };
        let serialized_payload = message.to_network_message()?;
        let header = MessageHeader {
            magin_network_nr: value.magic_number.into(),
            command: value.command.into(),
            payload_len: serialized_payload.len() as u32, //This payload will never be above u32
            checksum: calc_checksum(&serialized_payload),
        };
        Ok(Self {
            header: bincode::serialize(&header)?,
            message: serialized_payload,
        })
    }
}

impl From<SocketAddr> for NetworkAddress {
    fn from(addr: SocketAddr) -> Self {
        let (address, port) = match addr {
            SocketAddr::V4(addr) => (addr.ip().to_ipv6_mapped().segments(), addr.port()),
            SocketAddr::V6(addr) => (addr.ip().segments(), addr.port()),
        };

        let mut network_address: [u16; 8] = [0; 8];
        for (idx, net) in address.into_iter().enumerate() {
            network_address[idx] = htons(net);
        }

        Self {
            services: 0,
            addr: network_address,
            port: port.to_be(),
        }
    }
}

#[derive(Serialize)]
#[repr(C, packed)]
struct VersionMessage {
    version: i32,
    services: u64,
    timestamp: i64,
    recv_add: NetworkAddress,
    addr_from: NetworkAddress,
    nonce: u64,
    ua_len: u8,
    user_agent: [u8; 14], // Just hardoded, this app doesn't allow to set your own UA
    start_height: i32,
    relay: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[repr(C, packed)]
struct NetworkAddress {
    services: u64,
    addr: [u16; 8],
    port: u16,
}

impl ToNetworkMessage for VersionMessage {
    fn to_network_message(self) -> Result<Vec<u8>, Error> {
        Ok(bincode::serialize(&self)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_serialize_version() {
        let expected_data: Vec<u8> = vec![
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
        let str_addr = "79.116.148.118:8333".to_owned();
        let dest_address: SocketAddr = str_addr.parse().unwrap();

        let version_builder = VersionMessageBuilder::new(
            MessageMagicNumber::Main,
            dest_address,
            1693259353,
            0x6517e68c5db32e3b,
        );

        let btc_message: SerializedBitcoinMessage = version_builder.try_into().unwrap();
        let serialized_message: Vec<u8> = btc_message.to_network_message().unwrap();
        assert_eq!(serialized_message, expected_data);
    }
}
