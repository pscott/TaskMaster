use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process,
};
use taskmaster::{command::Command, config, DEFAULT_ADDR};

fn main() -> Result<(), std::io::Error> {
    let _config = config::parse().unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });


    let listener = TcpListener::bind(DEFAULT_ADDR)?;

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("{:?}", handle_connection(tcp_stream));
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let mut buf = [0; 1024];

    if let Ok(bytes) = stream.read(&mut buf) {
        let _cmd: Command = serde_json::from_str(&String::from_utf8_lossy(&buf[..bytes]))
            .map_err(|e| e.to_string())?;
        // Execute the command here.

        // Answer back to client with command's status.
        stream
            .write_all(b"Your program is running ok.")
            .map_err(|e| e.to_string())?;
    } else {
        eprintln!("Could not read from stream.");
    }
    Ok(())
}
