use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C)]
struct MessageHeader {
    pub magin_network_nr: [u8; 4],
    pub command: [u8; 12],
    pub payload_len: u32,
    pub checksum: u32,
}

impl Default for MessageHeader {
    fn default() -> Self {
        Self {
            magin_network_nr: [0xF9, 0xBE, 0xB4, 0xD9],
            command: [
                0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_len: Default::default(),
            checksum: Default::default(),
        }
    }
}

#[derive(Serialize)]
#[repr(C)]
struct BitcoinMessage {
    pub header: MessageHeader,
    pub message: VersionMessage,
}

#[derive(Serialize)]
#[repr(C)]
struct SerializedBitcoinMessage {
    pub header: Vec<u8>,
    pub message: Vec<u8>,
}

impl From<BitcoinMessage> for SerializedBitcoinMessage {
    fn from(message: BitcoinMessage) -> Self {
        let serialized_payload = message.message.to_network_message();
        let header = MessageHeader {
            magin_network_nr: message.header.magin_network_nr,
            command: message.header.command,
            payload_len: serialized_payload.len() as u32, //TODO: error handling,
            checksum: calc_checksum(&serialized_payload),
        };
        Self {
            header: bincode::serialize(&header).unwrap(),
            message: bincode::serialize(&message.message).unwrap(),
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

#[derive(Serialize)]
#[repr(C)]
enum Message {
    Version(VersionMessage),
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[repr(C, packed)]
struct NetworkAddress {
    pub services: u64,
    pub addr: [u16; 8],
    pub port: u16,
}
#[derive(Serialize)]
#[repr(C, packed)]
struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub recv_add: NetworkAddress,
    pub addr_from: NetworkAddress,
    pub nonce: u64,
    pub user_agent: [u8; 16], //&'static str, //now lets try just adding u8[] but later lets to string and impl
    //to Message/ To bytes
    pub start_height: i32,
    pub relay: bool,
}

trait ToNetworkMessage {
    fn to_network_message(&self) -> Vec<u8>;
}

impl ToNetworkMessage for VersionMessage {
    fn to_network_message(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

fn main() {
    let str_addr = "79.116.148.118:8333".to_owned();
    let dest_address: SocketAddr = str_addr.parse().unwrap();
    let (address, port) = match dest_address {
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

    let send_addr = NetworkAddress {
        services: 0,
        addr: address,
        port: htons(port),
    };

    let message = VersionMessage {
        version: 70001,
        services: 0,
        timestamp: chrono::offset::Utc::now().timestamp(),
        recv_add,
        addr_from: send_addr,
        nonce: 0x6517e68c5db32e3b,
        user_agent: [
            0x0f, 0x2A, 0x51, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x37, 0x2E,
            0x32, //first byte is len
            0x2E,
        ], // TODO: this is hack xD
        start_height: 0,
        relay: false,
    };

    let header = MessageHeader::default();
    let btc_message = BitcoinMessage { header, message };

    let btc_message: SerializedBitcoinMessage = btc_message.into();
    let concated: Vec<u8> = btc_message.to_network_message(); //;
    let mut stream = TcpStream::connect(dest_address)
        .map_err(|e| println!("{e}"))
        .unwrap();
    stream
        .set_read_timeout(None)
        .expect("set_read_timeout call failed");

    stream
        .write(&concated)
        .map_err(|e| println!("{e}"))
        .unwrap();
    let read_stream = stream.try_clone().unwrap();
    let mut stream_reader = std::io::BufReader::new(read_stream);

    let stream_reader_ref = &mut stream_reader;
    loop {
        let taken = stream_reader_ref.take(200);
        for byte in taken.bytes() {
            println!("{}", byte.unwrap());
        }
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
        let (address, port) = match dest_address {
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

        let send_addr = NetworkAddress {
            services: 0,
            addr: address,
            port: htons(port),
        };

        let message = VersionMessage {
            version: 70001,
            services: 0,
            timestamp: 1693259353,
            recv_add,
            addr_from: send_addr,
            nonce: 0x6517e68c5db32e3b,
            user_agent: [
                0x0f, 0x2A, 0x51, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x37, 0x2E,
                0x32, //first byte is len
                0x2E,
            ], // TODO: this is hack xD
            start_height: 0,
            relay: false,
        };

        let header = MessageHeader::default();
        let btc_message = BitcoinMessage { header, message };

        let btc_message: SerializedBitcoinMessage = btc_message.into();
        let serialized_message: Vec<u8> = btc_message.to_network_message();
        assert_eq!(serialized_message, expected_data);
    }
}
