#[derive(Debug, PartialEq)]
/// Command that will be executed.
pub enum Command<'a> {
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
    /// Reload the daemon’s configuration files, without add/remove (no restarts).
    ReRead,
    /// Restart multiple processes or groups.
    /// Note: restart does not reread config files. For that, see `Reread` and `Update`.
    Restart(&'a [&'a str]),
    /// Start one or multiple processes/groups.
    Start(&'a [&'a str]),
    /// Get status on one or multiple named processes.
    Status(&'a [&'a str]),
    /// Stop one or multiple processes or groups.
    Stop(&'a [&'a str]),
    /// Reload config and add/remove as necessary, and will restart affected programs.
    Update(&'a [&'a str]),
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
            Ok(Command::$name(&$args[1..]))
        } else {
            Err(ParsingError::MissingArguments)
        }
    };
    ($args:ident, $name:ident, unspecified) => {
        Ok(Command::$name(&$args[1..]))
    };
}

impl<'a> std::convert::TryFrom<&'a [&'a str]> for Command<'a> {
    type Error = ParsingError;

    fn try_from(args: &'a [&'a str]) -> Result<Self, Self::Error> {
        match args.get(0) {
            None => Err(Self::Error::EmptyCommand),
            Some(&command) => match command {
                "add" => create_command!(args, Add, multiple_args),
                "clear" => create_command!(args, Clear, multiple_args),
                "exit" => create_command!(args, Exit, zero_args),
                "pid" => create_command!(args, PID, unspecified),
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
