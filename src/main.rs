use error::Error;
use rand::Rng;
use std::env;
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

use config::Config;
use message_reader::MessageReader;
use messages::{version::VersionMessageBuilder, SerializedBitcoinMessage, ToNetworkMessage};

use crate::messages::{verack::VerackMessageBuilder, MessageCommand};

pub mod config;
pub mod error;
pub mod message_reader;
pub mod messages;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let config_file_name = if args.len() >= 2 {
        &args[1]
    } else {
        "config.json"
    };

    let config = Config::load_config(config_file_name)?;
    let dest_address: SocketAddr = config.dest_addr.parse()?;

    let mut rng = rand::thread_rng();
    let nonce: u64 = rng.gen();

    let version_builder = VersionMessageBuilder::new(
        config.network_type.clone(),
        dest_address,
        chrono::offset::Utc::now().timestamp(),
        nonce,
    );

    let btc_message: SerializedBitcoinMessage = version_builder.try_into()?;
    let serialized_message: Vec<u8> = btc_message.to_network_message()?;
    let mut stream = TcpStream::connect(dest_address)?;

    stream.set_read_timeout(None)?;

    println!("Sending Version message");
    stream.write_all(&serialized_message)?;
    println!("Message sent");

    let mut reader = MessageReader::new(Box::new(stream.try_clone()?));
    loop {
        let command = if let Some(command) = reader.read_message()? {
            command
        } else {
            continue;
        };
        println!("Received: {:?} message", &command);
        match command {
            MessageCommand::Version => {
                let verack_message = VerackMessageBuilder::new(config.network_type.clone());
                //TODO: oneliner ??
                let btc_message: SerializedBitcoinMessage = verack_message.try_into()?;
                let serialized_message: Vec<u8> = btc_message.to_network_message()?;

                println!("Sending Verack message");
                stream.write_all(&serialized_message)?;
                println!("Message sent");
            }
            MessageCommand::Verack => {
                println!("Hanshake with node: {:?} completed", dest_address);
                break;
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    run(args)
}
