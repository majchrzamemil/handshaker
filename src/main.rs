use std::{
    io::{BufReader, Read, Write},
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
//#[derive(Serialize, PartialEq, Debug)]
//#[repr(C)]
//enum Message {
//    Version(VersionMessage),
//}

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

fn main() {
    let str_addr = "79.116.148.118:8333".to_owned();
    let address: SocketAddr = str_addr.parse().unwrap();
    let (address, port) = match address {
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

    println!("{}", chrono::offset::Utc::now().timestamp());
    let message = VersionMessage {
        version: 70001,
        services: 0,
        timestamp: 1693243341, //chrono::offset::Utc::now().timestamp(), // 0x0000000050D0B211,
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

    let mut header = MessageHeader::default();
    let encoded_message: Vec<u8> = bincode::serialize(&message).unwrap();
    header.checksum = calc_checksum(&encoded_message);
    header.payload_len = encoded_message.len() as u32;
    let btc_message = BitcoinMessage {
        header,
        message,
    };
    let encoded_message: Vec<u8> = bincode::serialize(&btc_message).unwrap();
    println!("{:02X?}", encoded_message);

    //    //    //    //This is a test taken from example lets validate hexdumps
    //    //        let encoded_message = [
    //    //            0xf9, 0xbe, 0xb4, 0xd9, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00,
    //    //            0x00, 0x64, 0x00, 0x00, 0x00, 0x35, 0x8d, 0x49, 0x32, 0x62, 0xea, 0x00, 0x00, 0x01, 0x00,
    //    //            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0xb2, 0xd0, 0x50, 0x00, 0x00, 0x00, 0x00, 0x01,
    //    //            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //    //            0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //    //            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
    //    //            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0x2e, 0xb3, 0x5d, 0x8c, 0xe6, 0x17, 0x65, 0x0f,
    //    //            0x2f, 0x53, 0x61, 0x74, 0x6f, 0x73, 0x68, 0x69, 0x3a, 0x30, 0x2e, 0x37, 0x2e, 0x32, 0x2f,
    //    //            0xc0, 0x3e, 0x03, 0x00,
    //    //        ];
    //    //        println!("{:02X?}", encoded_message);
    //    //
    let mut stream = TcpStream::connect("79.116.148.118:8333")
        .map_err(|e| println!("{e}"))
        .unwrap();
    stream
        .set_read_timeout(None)
        .expect("set_read_timeout call failed");

    stream
        .write(encoded_message.as_slice())
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
