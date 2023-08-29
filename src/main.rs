use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use messages::{
    version::VersionMessageBuilder, MessageMagicNumber, SerializedBitcoinMessage, ToNetworkMessage,
};

use crate::messages::{verack::VerackMessageBuilder, MessageCommand, MessageHeader};

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

    let mut buf = [0x0; 1500]; //Just a big buffer, regular MTU
    loop {
        //read header 24
        let mut take = read_stream.try_clone().unwrap().take(24);
        let readed = take.read(&mut buf[0..24]).unwrap();
        if readed == 0 {
            continue;
        }
        let header: MessageHeader = bincode::deserialize(&buf[0..24]).unwrap(); //TODO: try_from
        let command: MessageCommand = match header.command.try_into() {
            Ok(command) => command,
            Err(_) => continue,
        };
        dbg!(&command);
        match command {
            MessageCommand::Version => {
                // Read the rest of payload
                let mut take = read_stream
                    .try_clone()
                    .unwrap()
                    .take(header.payload_len as u64);
                let _readed = take.read(&mut buf).unwrap(); //We discard that, in this
                                                            //implemementation we only respond to
                                                            //version message with verack
                let verack_message = VerackMessageBuilder::new(MessageMagicNumber::Main);
                let btc_message: SerializedBitcoinMessage = verack_message.into();
                let serialized_message: Vec<u8> = btc_message.to_network_message();
                stream
                    .write(&serialized_message)
                    .map_err(|e| println!("{e}"))
                    .unwrap();
            }
            MessageCommand::Verack => break,
        }
    }
}
