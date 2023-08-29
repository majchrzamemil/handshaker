use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

use config::Config;
use message_reader::MessageReader;
use messages::{version::VersionMessageBuilder, SerializedBitcoinMessage, ToNetworkMessage};

use crate::messages::{verack::VerackMessageBuilder, MessageCommand};

pub mod config;
pub mod message_reader;
pub mod messages;

fn main() {
    let config = Config::load_config().unwrap();
    let dest_address: SocketAddr = config.dest_addr.parse().unwrap();

    let version_builder = VersionMessageBuilder::new(
        config.network_type.clone(),
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

    let mut reader = MessageReader::new(Box::new(stream.try_clone().unwrap()));
    loop {
        let command = if let Some(command) = reader.read_message() {
            command
        } else {
            continue;
        };
        dbg!(&command);
        match command {
            MessageCommand::Version => {
                // Read the rest of payload
                let verack_message = VerackMessageBuilder::new(config.network_type.clone());
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
