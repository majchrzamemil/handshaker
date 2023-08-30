use rand::Rng;
use std::{
    env,
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
    let args: Vec<String> = env::args().collect();
    let config_file_name = if args.len() >= 2 {
        &args[1]
    } else {
        "config.json"
    };

    let config = Config::load_config(config_file_name).unwrap();
    let dest_address: SocketAddr = config.dest_addr.parse().unwrap();

    let mut rng = rand::thread_rng();
    let nonce: u64 = rng.gen();

    let version_builder = VersionMessageBuilder::new(
        config.network_type.clone(),
        dest_address,
        chrono::offset::Utc::now().timestamp(),
        nonce,
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
    println!("Sending Version message");

    let mut reader = MessageReader::new(Box::new(stream.try_clone().unwrap()));
    loop {
        let command = if let Some(command) = reader.read_message() {
            command
        } else {
            continue;
        };
        println!("Received: {:?} message", &command);
        match command {
            MessageCommand::Version => {
                let verack_message = VerackMessageBuilder::new(config.network_type.clone());
                let btc_message: SerializedBitcoinMessage = verack_message.into();
                let serialized_message: Vec<u8> = btc_message.to_network_message();
                println!("{:#04X?}", &serialized_message);
                stream
                    .write(&serialized_message)
                    .map_err(|e| println!("{e}"))
                    .unwrap();
                println!("Sending Verack message");
            }
            MessageCommand::Verack => {
                println!("Hanshake with node: {:?} completed", dest_address);
                break;
            }
        }
    }
}
