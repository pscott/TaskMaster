use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// Command that will be executed.
pub enum Command {
    /// Activates any updates in config for process/group.
    Add(Vec<String>),
    /// Clear one or multiple process’ log files.
    Clear(Vec<String>),
    /// Exit taskmasterctl.
    Exit,
    /// Get the `PID` of one or multiple child processes.
    Pid(Vec<String>),
    /// Removes process/group from active config.
    Remove(Vec<String>),
    /// Reload the daemon’s configuration files, without add/remove (no restarts).
    ReRead,
    /// Restart multiple processes or groups.
    /// Note: restart does not reread config files. For that, see `Reread` and `Update`.
    Restart(Vec<String>),
    /// Start one or multiple processes/groups.
    Start(Vec<String>),
    /// Get status on one or multiple named processes.
    Status(Vec<String>),
    /// Stop one or multiple processes or groups.
    Stop(Vec<String>),
    /// Reload config and add/remove as necessary, and will restart affected programs.
    Update(Vec<String>),
}

#[derive(Debug, PartialEq)]
/// Errors that could appear when one tries to parse an input into a Command.
pub enum ParsingError {
    EmptyCommand,
    UnknownCommand(String),
    UnexpectedArguments,
    MissingArguments,
}

impl ParsingError {
    pub fn display(&self) {
        match self {
            Self::UnknownCommand(s) => eprintln!("Unknown command: {}", s),
            Self::UnexpectedArguments => eprintln!("Unexpected arguments"),
            Self::MissingArguments => eprintln!("Missing arguments"),
            _ => {}
        }
    }
}

/// Creates a new `Command` based on the arguments provided.
/// Example:
/// ```rust
/// create_command!(args, Exit, zero_args);
/// ```
/// Will create a `Command::Exit`, and will error if the number of
/// additional arguments (after the first argument) is not 0.
/// Possible values are: zero_args, multiple_args, unspecified.
macro_rules! create_command {
    ($args:ident, $name:ident, zero_args) => {
        if $args.len() == 1 {
            Ok(Command::$name)
        } else {
            Err(ParsingError::UnexpectedArguments)
        }
    };
    ($args:ident, $name:ident, multiple_args) => {
        if $args.len() > 1 {
            Ok(Command::$name(
                $args[1..]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            ))
        } else {
            Err(ParsingError::MissingArguments)
        }
    };
    ($args:ident, $name:ident, unspecified) => {
        Ok(Command::$name(
            $args[1..]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        ))
    };
}

impl<'a> std::convert::TryFrom<&[&str]> for Command {
    type Error = ParsingError;

    fn try_from(args: &[&str]) -> Result<Self, Self::Error> {
        match args.get(0) {
            None => Err(Self::Error::EmptyCommand),
            Some(&command) => match command {
                "add" => create_command!(args, Add, multiple_args),
                "clear" => create_command!(args, Clear, multiple_args),
                "exit" => create_command!(args, Exit, zero_args),
                "pid" => create_command!(args, Pid, unspecified),
                "remove" => create_command!(args, Remove, multiple_args),
                "reread" => create_command!(args, ReRead, zero_args),
                "restart" => create_command!(args, Restart, multiple_args),
                "start" => create_command!(args, Start, multiple_args),
                "status" => create_command!(args, Status, unspecified),
                "stop" => create_command!(args, Stop, multiple_args),
                "update" => create_command!(args, Update, multiple_args),
                other => Err(Self::Error::UnknownCommand(other.into())),
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::shadow_unrelated)] // We don't mind shadowing here, so stop Clippy from complaining.
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn empty_line() {
        let args: &[&str] = &[];
        let res = Command::try_from(args);
        assert_eq!(res, Err(ParsingError::EmptyCommand));
    }

    #[test]
    fn unknwon_command() {
        let args: &[&str] = &["42"];
        let res = Command::try_from(args);
        assert_eq!(res, Err(ParsingError::UnknownCommand(args[0].into())));
    }

    #[test]
    fn typo() {
        let args: &[&str] = &["addd"];
        let res = Command::try_from(args);
        assert_eq!(res, Err(ParsingError::UnknownCommand(args[0].into())));
    }

    #[test]
    fn zero_args_command() {
        let args: &[&str] = &["exit"];
        let res = Command::try_from(args);
        assert!(res.is_ok());

        let args: &[&str] = &["exit", "123"];
        let res = Command::try_from(args);
        assert_eq!(res, Err(ParsingError::UnexpectedArguments));
    }

    #[test]
    fn multiple_args_command() {
        let args: &[&str] = &["clear"];
        let res = Command::try_from(args);
        assert_eq!(res, Err(ParsingError::MissingArguments));

        let args: &[&str] = &["clear", "123"];
        let res = Command::try_from(args);
        assert!(res.is_ok());
    }

    #[test]
    fn unspecified() {
        let args: &[&str] = &["pid"];
        let res = Command::try_from(args);
        assert!(res.is_ok());

        let args: &[&str] = &["pid", "123"];
        let res = Command::try_from(args);
        assert!(res.is_ok());
    }

    #[test]
    fn one_arg_unspecified() {
        let args: &[&str] = &["pid", "cat"];
        let res = Command::try_from(args);
        assert!(res.is_ok());
    }

    #[test]
    fn two_args_unspecified() {
        let args: &[&str] = &["status", "cat", "nginx"];
        let res = Command::try_from(args);
        assert!(res.is_ok());
    }

    #[test]
    fn supported_commands() {
        let lines: &[&[&str]] = &[
            &["add", "cat"],
            &["clear", "python"],
            &["exit"],
            &["pid", "cat"],
            &["remove", "cat"],
            &["reread"],
            &["restart", "cat"],
            &["start", "cat"],
            &["status", "cat", "nginx", "top"],
            &["stop", "cat", "nginx"],
            &["update", "cat", "ft_server"],
        ];
        for &line in lines {
            let res = Command::try_from(line);
            dbg!(&res);
            assert!(res.is_ok());
        }
    }
}
