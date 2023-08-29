use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

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
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

impl VersionMessageBuilder {
    pub fn new(magic_number: MessageMagicNumber, addr_recv: SocketAddr, timestamp: i64) -> Self {
        Self {
            magic_number,
            command: MessageCommand::Version,
            version: 70001,
            timestamp,
            addr_recv,
            addr_from: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            nonce: 0x6517e68c5db32e3b, //TODO: change for rand later
            user_agent: String::new(), //TODO: handler UA later
            start_height: 0,
            relay: false,
        }
    }
}

impl From<VersionMessageBuilder> for SerializedBitcoinMessage {
    fn from(value: VersionMessageBuilder) -> Self {
        let (address, port) = match value.addr_recv {
            SocketAddr::V4(addr) => (addr.ip().to_ipv6_mapped().segments(), addr.port()),
            SocketAddr::V6(addr) => (addr.ip().segments(), addr.port()),
        };

        let mut network_address: [u16; 8] = [0; 8];
        for (idx, net) in address.into_iter().enumerate() {
            network_address[idx] = htons(net);
        }
        let recv_add = NetworkAddress {
            services: 0,
            addr: network_address,
            port: htons(port),
        };
        let my_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        let (address, port) = match my_addr {
            SocketAddr::V4(addr) => (addr.ip().to_ipv6_mapped().segments(), addr.port()),
            SocketAddr::V6(addr) => (addr.ip().segments(), addr.port()),
        };

        let addr_from = NetworkAddress {
            services: 0,
            addr: address,
            port: port.to_be(),
        };
        let message = VersionMessage {
            version: value.version,
            services: 0,
            timestamp: value.timestamp,
            recv_add,
            addr_from,
            nonce: value.nonce,
            ua_len: 0x0F,
            user_agent: [
                0x2A, 0x51, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x37, 0x2E, 0x32,
                0x2E,
            ],
            start_height: value.start_height,
            relay: value.relay,
        };
        let serialized_payload = message.to_network_message();
        let header = MessageHeader {
            magin_network_nr: value.magic_number.into(),
            command: value.command.into(),
            payload_len: serialized_payload.len() as u32, //TODO: error handling,
            checksum: calc_checksum(&serialized_payload),
        };
        Self {
            header: bincode::serialize(&header).unwrap(),
            message: bincode::serialize(&message).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[repr(C, packed)]
struct NetworkAddress {
    services: u64,
    addr: [u16; 8],
    port: u16,
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
    user_agent: [u8; 15], //&'static str, //now lets try just adding u8[] but later lets to string and impl
    //o Message/ To bytes
    start_height: i32,
    relay: bool,
}

impl ToNetworkMessage for VersionMessage {
    fn to_network_message(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_serialize_version() {
        let expected_data = vec![
            0xF9, 0xBE, 0xB4, 0xD9, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x65, 0x00, 0x00, 0x00, 0x0E, 0xC1, 0x1F, 0x4F, 0x71, 0x11, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x59, 0x16, 0xED, 0x64, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x4F, 0x74, 0x94, 0x76, 0x20, 0x8D,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3B, 0x2E,
            0xB3, 0x5D, 0x8C, 0xE6, 0x17, 0x65, 0x0F, 0x2A, 0x51, 0x61, 0x74, 0x6F, 0x73, 0x68,
            0x69, 0x3A, 0x30, 0x2E, 0x37, 0x2E, 0x32, 0x2E, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let str_addr = "79.116.148.118:8333".to_owned();
        let dest_address: SocketAddr = str_addr.parse().unwrap();

        let version_builder =
            VersionMessageBuilder::new(MessageMagicNumber::Main, dest_address, 1693259353);

        let btc_message: SerializedBitcoinMessage = version_builder.into();
        let serialized_message: Vec<u8> = btc_message.to_network_message();
        assert_eq!(serialized_message, expected_data);
    }
}
