use serde_json::Deserializer;
use std::convert::TryFrom;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use taskmaster::command::Command;
use taskmaster::config::Config;
use taskmaster::DEFAULT_ADDR;

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
    let objects = Deserializer::from_reader(stream).into_iter::<Command>();

    for obj in objects {
        println!("{:?}", obj);
    }
}
