use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use messages::{
    version::VersionMessageBuilder, MessageMagicNumber, SerializedBitcoinMessage, ToNetworkMessage,
};

use crate::messages::{MessageCommand, MessageHeader};

pub mod messages;

fn main() {
    let str_addr = "79.116.148.118:8333".to_owned();
    let dest_address: SocketAddr = str_addr.parse().unwrap();

    let version_builder = VersionMessageBuilder::new(
        MessageMagicNumber::Main,
        dest_address,
        chrono::offset::Utc::now().timestamp(),
    );

    let btc_message: SerializedBitcoinMessage = version_builder.into();
    let serialized_message: Vec<u8> = btc_message.to_network_message();
    let mut stream = TcpStream::connect(dest_address)
        .map_err(|e| println!("{e}"))
        .unwrap();
    stream
        .set_read_timeout(None)
        .expect("set_read_timeout call failed");

    stream
        .write(&serialized_message)
        .map_err(|e| println!("{e}"))
        .unwrap();
    let read_stream = stream.try_clone().unwrap();

    let mut buf = [0x0; 2000];
    loop {
        //read header 24
        //from header read payload_len and read payload
        //coninue
        let mut take = read_stream.try_clone().unwrap().take(24);
        let readed = take.read(&mut buf[0..24]).unwrap();
        if readed == 0 {
            continue;
        }
        let header: MessageHeader = bincode::deserialize(&buf[0..24]).unwrap(); //TODO: try_from
        let command: MessageCommand = header.command.try_into().unwrap();
        dbg!(command);
        break;
    }
}
