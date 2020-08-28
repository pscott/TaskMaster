extern crate liner;
mod command;
mod config;

use command::Command;
use config::Config;
use liner::{Completer, Context};
use std::convert::TryFrom;
use std::path::Path;

/// Placeholder struct for Completer.
struct EmptyCompleter;

impl Completer for EmptyCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}

fn main() -> Result<(), std::io::Error> {
    let path = Path::new("config.yaml");
    let _config = Config::try_from(path)?;

    let mut con = Context::new();

    loop {
        let line = con.read_line("$ ", None, &mut EmptyCompleter)?;

        let args = line.split_ascii_whitespace().collect::<Vec<&str>>();
        let cmd = Command::try_from(&args[..]);
        match cmd {
            Ok(Command::Exit) => break,
            Ok(_) => {} // We will need to actually execute the command here!
            Err(cmd_err) => cmd_err.display(),
        }

        con.history.push(line.into())?;
    }
    Ok(())
}
