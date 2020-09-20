use serde_json::Deserializer;
use std::{
    convert::TryFrom,
    net::{TcpListener, TcpStream},
    path::Path,
};
use taskmaster::{command::Command, config::Config, DEFAULT_ADDR};

fn main() -> Result<(), std::io::Error> {
    let path = Path::new("config.yaml");
    let _config = Config::try_from(path)?;
    let listener = TcpListener::bind(DEFAULT_ADDR)?;

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => handle_connection(tcp_stream),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) {
    let commands = Deserializer::from_reader(stream).into_iter::<Command>();

    for cmd in commands {
        println!("{:?}", cmd);
    }
}
