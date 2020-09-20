use liner::{Completer, Context};
use std::{convert::TryFrom, net::TcpStream};
use taskmaster::{command::Command, DEFAULT_ADDR};

/// Prompt displayed when using taskmaster in interactive mode.
const TASKMASTER_PROMPT: &str = "taskmaster> ";

/// Placeholder struct for Completer.
struct EmptyCompleter;

impl Completer for EmptyCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}
fn main() -> Result<(), String> {
    let mut con = Context::new();
    let stream = TcpStream::connect(DEFAULT_ADDR).map_err(|_| {
        "Could not connect to the daemon. You can start the daemon by typing `taskmasterd`"
            .to_string()
    })?;

    loop {
        let line = con
            .read_line(TASKMASTER_PROMPT, None, &mut EmptyCompleter)
            .map_err(|e| e.to_string())?;

        let args = line.split_ascii_whitespace().collect::<Vec<&str>>();
        let cmd = Command::try_from(&args[..]);
        match cmd {
            Ok(Command::Exit) => break,
            Ok(command) => {
                serde_json::to_writer(&stream, &command).map_err(|e| e.to_string())?;
            }
            Err(cmd_err) => cmd_err.display(),
        }

        con.history.push(line.into()).map_err(|e| e.to_string())?;
    }
    Ok(())
}
