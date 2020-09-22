use crate::{command::Command, config::Config, threadpool::ThreadPool, DEFAULT_ADDR};
use std::io::{Read, Write};
use std::{
    convert::TryFrom,
    net::{TcpListener, TcpStream},
    path::Path,
};

/// Number of threads in the `ThreadPool`.
const NUM_THREADS: usize = 4;

/// Runs the server.
///
/// # Errors
///
/// Errors if it parsing the config file errors, or if binding to the default address fails.
pub fn run() -> Result<(), std::io::Error> {
    let path = Path::new("config.yaml");
    let _config = Config::try_from(path)?;

    let listener = TcpListener::bind(DEFAULT_ADDR)?;

    let pool = ThreadPool::new(NUM_THREADS);

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => pool.execute(|| {
                let _ = handle_connection(tcp_stream);
            }),
            Err(e) => eprintln!("Error while listening for incoming messages: {:?}", e),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let mut buf = [0; 1024];

    match stream.read(&mut buf) {
        Ok(bytes) => {
            let _cmd: Command = serde_json::from_str(&String::from_utf8_lossy(&buf[..bytes]))
                .map_err(|e| format! {"Failed to deserialize Command: {:?}", e})?;
            // Execute the command here.

            // Answer back to client with command's status.
            stream
                .write_all(b"Your program is running ok.")
                .map_err(|e| e.to_string())?;
        }
        Err(e) => {
            eprintln!("Could not read from stream: {:?}", e);
        }
    }
    Ok(())
}
