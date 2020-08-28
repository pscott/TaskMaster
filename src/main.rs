extern crate liner;
mod config;

use config::Config;
use liner::{Completer, Context};
use std::convert::TryFrom;
use std::path::Path;

#[derive(Debug)]
/// Command that will be executed.
enum Command<'a> {
    /// Activates any updates in config for process/group.
    Add(&'a [&'a str]),
    /// Clear one or multiple process’ log files
    Clear(&'a [&'a str]),
    /// Exit TaskMaster.
    Exit,
    /// Get the PID of one or multiple child processes.
    PID(&'a [&'a str]),
    /// Removes process/group from active config.
    Remove(&'a [&'a str]),
    /// Restart multiple processes or groups.
    /// Note: restart does not reread config files. For that, see reread and update.
    Restart(&'a [&'a str]),
    /// Start one or multiple processes/groups.
    Start(&'a [&'a str]),
    /// Get status on one or multiple named processes.
    Status(&'a str),
    /// Stop one or multiple processes or groups.
    Stop(&'a [&'a str]),
    /// Reload config and add/remove as necessary, and will restart affected programs.
    Update(&'a [&'a str]),
}

#[derive(Debug)]
/// Errors that could appear when one tries to parse an input into a Command.
enum CommandParsingError {
    EmptyCommand,
    UnknownCommand(String),
    UnexpectedArguments,
    MissingArguments,
}

impl CommandParsingError {
    fn display(&self) {
        match self {
            Self::UnknownCommand(s) => eprintln!("{}", s),
            Self::UnexpectedArguments => eprintln!("Unexpected arguments"),
            Self::MissingArguments => eprintln!("Missing arguments"),
            _ => {}
        }
    }
}

/// Creates a new `Command` based on the arguments provided.
/// Example:
/// ```rust
/// create_command!(args, Exit, zero_arg);
/// ```
/// Will create a `Command::Exit`, and will error if the number of
/// additional arguments (after the first argument) is not 0.
/// Possible values are: zero_arg, one_arg, multiple_args.
macro_rules! create_command {
    ($args:ident, $name:ident, zero_arg) => {
        if $args.len() == 1 {
            Ok(Command::$name)
        } else {
            Err(CommandParsingError::UnexpectedArguments)
        }
    };
    ($args:ident, $name:ident, one_arg) => {
        if $args.len() == 2 {
            Ok(Command::$name($args[1]))
        } else if $args.len() < 2 {
            Err(CommandParsingError::MissingArguments)
        } else {
            Err(CommandParsingError::UnexpectedArguments)
        }
    };
    ($args:ident, $name:ident, multiple_args) => {
        if $args.len() > 1 {
            Ok(Command::$name(&$args[1..]))
        } else {
            Err(CommandParsingError::MissingArguments)
        }
    };
}

impl<'a> std::convert::TryFrom<&'a [&'a str]> for Command<'a> {
    type Error = CommandParsingError;

    fn try_from(data: &'a [&'a str]) -> Result<Self, Self::Error> {
        match data.get(0) {
            None => Err(Self::Error::EmptyCommand),
            Some(&command) => match command {
                "add" => create_command!(data, Add, multiple_args),
                "clear" => create_command!(data, Clear, multiple_args),
                "exit" => create_command!(data, Exit, zero_arg),
                "pid" => create_command!(data, PID, multiple_args),
                "remove" => create_command!(data, Remove, multiple_args),
                "restart" => create_command!(data, Restart, multiple_args),
                "start" => create_command!(data, Start, multiple_args),
                "status" => create_command!(data, Status, one_arg),
                "stop" => create_command!(data, Stop, multiple_args),
                "update" => create_command!(data, Update, multiple_args),
                other => Err(Self::Error::UnknownCommand(format!(
                    "Unknown command: {}",
                    other
                ))),
            },
        }
    }
}

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
