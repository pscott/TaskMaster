mod config;

use config::Config;
use std::convert::TryFrom;
use std::path::Path;

extern crate liner;
use liner::{Completer, Context};

struct EmptyCompleter;

impl Completer for EmptyCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug)]
enum Command<'a> {
    Clear(&'a [&'a str]),
    ClearAll,
    Exit,
    Restart(&'a [&'a str]),
    RestartAll,
    Start(&'a [&'a str]),
    StartAll,
    Status,
    Stop(&'a [&'a str]),
    StopAll,
    Update,
    UpdateAll,
}

#[derive(Debug)]
enum CommandError {
    EmptyCommand,
    UnknownCommand(String),
    UnexpectedArguments,
    MissingArguments,
}

impl CommandError {
    fn display(&self) {
        match self {
            Self::UnknownCommand(s) => eprintln!("{}", s),
            Self::UnexpectedArguments => eprintln!("Unexpected arguments."),
            Self::MissingArguments => eprintln!("Missing arguments."),
            _ => {},
        }
    }
}

impl<'a> std::convert::TryFrom<&'a [&'a str]> for Command<'a> {
    type Error = CommandError;

    // TODO add a macro to avoid code duplication
    fn try_from(data: &'a [&'a str]) -> Result<Self, Self::Error> {
        match data.get(0) {
            Some(&command) => match command {
                "clear" => {
                    if data[1..].is_empty() {
                        Err(Self::Error::MissingArguments)
                    } else if data[1] == "all" {
                        Ok(Self::ClearAll)
                    } else {
                        Ok(Self::Clear(&data[1..]))
                    }
                }
                "exit" => {
                    if data[1..].is_empty() {
                        Ok(Self::Exit)
                    } else {
                        Err(Self::Error::UnexpectedArguments)
                    }
                }
                "restart" => {
                    if data[1..].is_empty() {
                        Err(Self::Error::MissingArguments)
                    } else if data[1] == "all" {
                        Ok(Self::RestartAll)
                    } else {
                        Ok(Self::Restart(&data[1..]))
                    }
                }
                "start" => {
                    if data[1..].is_empty() {
                        Err(Self::Error::MissingArguments)
                    } else if data[1] == "all" {
                        Ok(Self::StartAll)
                    } else {
                        Ok(Self::Start(&data[1..]))
                    }
                }
                "status" => {
                    if data[1..].is_empty() {
                        Ok(Self::Status)
                    } else {
                        Err(Self::Error::UnexpectedArguments)
                    }
                }
                "stop" => {
                    if data[1..].is_empty() {
                        Err(Self::Error::MissingArguments)
                    } else if data[1] == "all" {
                        Ok(Self::StopAll)
                    } else {
                        Ok(Self::Stop(&data[1..]))
                    }
                }
                "update" => {
                    if data[1..].is_empty() {
                        Ok(Self::Update)
                    } else if data[1] == "all" {
                        Ok(Self::UpdateAll)
                    } else {
                        Err(Self::Error::UnexpectedArguments)
                    }
                }
                other => Err(Self::Error::UnknownCommand(format!(
                    "Unknown command: {}",
                    other
                ))),
            },
            None => Err(Self::Error::EmptyCommand),
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let path = Path::new("config.yaml");
    let _config = Config::try_from(path)?;
    let mut con = Context::new();

    loop {
        let res = con.read_line("$ ", None, &mut EmptyCompleter)?;

        let words = res.split_ascii_whitespace().collect::<Vec<&str>>();
        let cmd = Command::try_from(&words[..]);
        match cmd {
            Ok(Command::Exit) => break,
            Ok(_) => {},
            Err(cmd_err) => cmd_err.display(),
        }

        con.history.push(res.into())?;
    }
    Ok(())
}
