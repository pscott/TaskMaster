mod command;
mod config;

use command::Command;
use liner::{Completer, Context};
use std::{convert::TryFrom, path::Path, process};

/// Placeholder struct for Completer.
struct EmptyCompleter;

impl Completer for EmptyCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}

fn main() -> Result<(), std::io::Error> {
    let path = Path::new("config.yaml");
    let config = config::parse(path);
    #[cfg(debug_assertions)]
    if let Ok(config) = config {
        println!("{:?}", config);
    } else {
        println!("Parsing error! Configuration file does not respect taskmasterctl/yaml format.");
        process::exit(1);
    };
    #[cfg(not(debug_assertions))]
    if let Ok(config) = config { } else {
        println!("Parsing error! Configuration file does not respect taskmasterctl/yaml format.");
        process::exit(1);
    };

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
