use crate::{command::Command, DEFAULT_ADDR};
use liner::{Completer, Context};
use std::io::{Read, Write};
use std::{convert::TryFrom, net::TcpStream};

/// Prompt displayed when using taskmaster in interactive mode.
const TASKMASTER_PROMPT: &str = "taskmaster> ";

/// Placeholder struct for Completer.
struct EmptyCompleter;

impl Completer for EmptyCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}

/// Runs the client.
///
/// # Errors
///
/// Returns an error if the stream fails to open, or if there's an error while reading stdin.
pub fn run() -> Result<(), String> {
    let mut con = Context::new();

    // Try connecting to the daemon to make sure it's running.
    {
        let _stream = TcpStream::connect(DEFAULT_ADDR).map_err(|_| {
            "Could not connect to the daemon. You can start the daemon by typing `taskmasterd`"
                .to_string()
        })?;
    }

    loop {
        let line = con
            .read_line(TASKMASTER_PROMPT, None, &mut EmptyCompleter)
            .map_err(|e| e.to_string())?;

        let args = line.split_ascii_whitespace().collect::<Vec<&str>>();
        let cmd = Command::try_from(&args[..]);
        match cmd {
            Ok(Command::Exit) => break,
            Ok(command) => match serde_json::to_string(&command) {
                Ok(message) => {
                    // Open up the stream to communicate with the daemon.
                    let mut stream = TcpStream::connect(DEFAULT_ADDR).map_err(|_| {
        			    "Could not connect to the daemon. You can start the daemon by typing `taskmasterd`"
        			        .to_string()
					})?;

                    if let Err(e) = stream.write(message.as_bytes()) {
                        eprintln!("Could not sent message: {:?}", e);
                    } else {
                        // Message got sent correctly.
                        let mut res = String::new();

                        // Read back answer from server.
                        stream.read_to_string(&mut res).map_err(|e| e.to_string())?;
                        println!("response: {}", res);
                    }
                }
                Err(e) => eprintln!("Could not serialize command: {:?}", e),
            },
            Err(e) => e.display(),
        }

        con.history
            .push(line.into())
            .unwrap_or_else(|e| eprintln!("Failed to write to history: {}", e));
    }
    Ok(())
}
