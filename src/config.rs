//! # Config
//!
//! Library parsing the taskmasterd and taskmasterctl configuration files.
//! Ref http://supervisord.org/configuration.html#unix-http-server-section-settings
//! Documentation is an almost copy-paste of the supervisor official documentation.
//! Thanks to them!
//!
//! Taskmasterd config module aims at parsing yaml and Windows-INI-style format.
//! This way it follows the trend of having yaml configuration file and backward compatibility
//! of supervisord.conf file in ini format making easier for users to shift from Supervisor to Taskmaster.
//!
//! Environment Variables
//! Environment variables that are present in the environment at the time that supervisord
//! is started can be used in the configuration file using the Python string expression syntax %(ENV_X)s:
//!
//![program:example]
//!command=/usr/bin/example --loglevel=%(ENV_LOGLEVEL)s
//!
//!In the example above, the expression %(ENV_LOGLEVEL)s would be expanded to the value of the environment variable LOGLEVEL.
//!
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
};

/// Restart conditions for a service.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum Restart {
    Never,
    Always,
    Unexpected,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    programs: Option<HashMap<String, Program>>,
    taskmasterd: Option<Taskmasterd>,
    taskmasterctl: Option<Taskmasterctl>,
    unix_http_server: Option<UnixHttpServer>,
    inet_http_server: Option<InetHttpServer>,
    include: Option<Include>,
    group: Option<HashMap<String, Group>>,
    fcgi_program: Option<HashMap<String, FcgiProgram>>,
    eventlistener: Option<HashMap<String, EventListener>>,
    rpcinterface: Option<HashMap<String, RpcInterface>>,
}

/// Program structure is a section of Config in order to run a task.
/// The configuration file must contain one or more program sections
/// in order for taskmasterd to know which programs it should start and control.
/// The header value is composite value.
/// It is the word “program”, followed directly by a colon, then the program name.
/// A header value of [program:foo] describes a program with the name of “foo”.
/// The name is used within client applications that control the processes that
/// are created as a result of this configuration.
/// It is an error to create a program section that does not have a name.
/// The name must not include a colon character or a bracket character.
/// The value of the name is used as the value for the %(program_name)s string
/// expression expansion within other values where specified.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Program {
    /// The command that will be run when this program is started.
    /// The command can be either absolute (e.g. /path/to/programname) or relative (e.g. programname).
    /// If it is relative, the taskmasterd’s environment $PATH will be searched for the executable.
    /// Programs can accept arguments, e.g. /path/to/program foo bar.
    /// The command line can use double quotes to group arguments with spaces in them to pass to the program,
    /// e.g. /path/to/program/name -p "foo bar".
    /// Note that the value of command may include Python string expressions,
    /// e.g. /path/to/programname --port=80%(process_num)02d might expand to /path/to/programname --port=8000 at runtime.
    /// String expressions are evaluated against a dictionary containing the keys
    /// group_name, host_node_name, program_name, process_num, numprocs,
    /// here (the directory of the taskmasterd config file), and all taskmasterd’s environment variables prefixed
    /// with ENV_. Controlled programs should themselves not be daemons,
    /// as taskmasterd assumes it is responsible for daemonizing its subprocesses (see Nondaemonizing of Subprocesses).
    /// Default: No default
    /// Required: Yes
    command: String, // (PathBuf, String, ...)

    /// A Python string expression that is used to compose the taskmaster process name for this process.
    /// You usually don’t need to worry about setting this unless you change numprocs.
    /// The string expression is evaluated against a dictionary that includes
    /// group_name, host_node_name, process_num, program_name, and here (the directory of the taskmasterd config file).
    /// Default: %(program_name)s
    /// Required: No
    process_name: Option<String>,

    /// Taskmaster will start as many instances of this program as named by numprocs.
    /// Note that if numprocs > 1, the process_name expression must include %(process_num)s
    /// (or any other valid Python string expression that includes process_num) within it.
    /// Default: 1
    /// Required: No
    numprocs: Option<u16>,

    /// An integer offset that is used to compute the number at which numprocs starts.
    /// Default: 0
    /// Required: No
    numprocs_start: Option<u16>,

    /// The relative priority of the program in the start and shutdown ordering.
    /// Lower priorities indicate programs that start first and shut down last at
    /// startup and when aggregate commands are used in various clients
    /// (e.g. “start all”/”stop all”).
    /// Higher priorities indicate programs that start last and shut down first.
    /// Default: `999`
    /// Required: No
    priority: Option<i32>,

    /// If true, this program will start automatically when taskmasterd is started.
    /// Default: `true`
    /// Required: No
    autostart: Option<bool>,

    /// The total number of seconds which the program needs to stay running after a startup
    /// to consider the start successful (moving the process from the STARTING state to the RUNNING state).
    /// Set to 0 to indicate that the program needn’t stay running for any particular amount of time.
    /// Default: `1`
    /// Required: No
    startsecs: Option<i32>,

    /// The number of serial failure attempts that taskmasterd will allow when attempting to start
    /// the program before giving up and putting the process into an FATAL state.
    /// Default: `3`
    /// Required: No
    startretries: Option<i32>,

    /// Specifies if taskmasterd should automatically restart a process if it exits when it is
    /// in the RUNNING state.
    /// May be one of `false`, `unexpected`, or `true`.
    /// If `false`, the process will not be autorestarted.
    /// If `unexpected`, the process will be restarted when the program exits with an
    /// exit code that is not one of the exit codes associated with this process’ configuration.
    /// If `true`, the process will be unconditionally restarted when it exits, without regard to its exit code.
    /// Default: `unexpected`
    /// Required: No.
    autorestart: Option<Restart>,

    /// The list of “expected” exit codes for this program used with autorestart.
    /// If the autorestart parameter is set to `unexpected`, and the process exits
    /// in any other way than as a result of a tadskmaster stop request,
    /// taskmasterd will restart the process if it exits with an exit code that is not defined in this list.
    /// Default: `0`
    /// Required: No
    exitcodes: Option<Vec<i32>>,

    /// The signal used to kill the program when a stop is requested.
    /// This can be any of `TERM`, `HUP`, `INT`, `QUIT`, `KILL`, `USR1`, or `USR2`.
    /// Default: `TERM`
    /// Required: No
    stopsignal: Option<Vec<String>>,

    /// The number of seconds to wait for the OS to return a `SIGCHLD` to taskmasterd after the program
    /// has been sent a stopsignal.
    /// If this number of seconds elapses before taskmasterd receives a SIGCHLD from the process,
    /// taskmasterd will attempt to kill it with a final `SIGKILL`.
    /// Default: `10`
    /// Required: No
    stopwaitsecs: Option<i32>,

    /// If `true`, the flag causes taskmaster to send the stop signal to the whole process group
    /// and implies killasgroup is `true`.
    /// This is useful for programs, such as Flask in debug mode, that do not propagate
    /// stop signals to their children, leaving them orphaned.
    /// Default: `false`
    /// Required: No
    stopasgroup: Option<bool>,

    /// If true, when resorting to send `SIGKILL` to the program to terminate it send it to its
    /// whole process group instead, taking care of its children as well,
    /// useful e.g with Python programs using multiprocessing.
    /// Default: `false`
    /// Required: No
    killasgroup: Option<bool>,

    /// Instruct taskmasterd to use this UNIX user account as the account which runs the program.
    /// The user can only be switched if taskmasterd is run as the root user.
    /// If taskmasterd can’t switch to the specified user, the program will not be started.
    /// Default: Do not switch users
    /// Required: No
    user: Option<String>,

    /// If `true`, cause the process’ stderr output to be sent back to taskmasterd on its
    /// stdout file descriptor
    /// (in UNIX shell terms, this is the equivalent of executing /the/program 2>&1).
    /// Default: `false`
    /// Required: No
    redirect_stderr: Option<bool>,

    /// Put process stdout output in this file (and if redirect_stderr is true, also place
    /// stderr output in this file).
    /// If stdout_logfile is unset or set to `AUTO`, taskmaster will automatically choose a file location.
    /// If this is set to `NONE`, taskmasterd will create no log file.
    /// `AUTO` log files and their backups will be deleted when taskmasterd restarts.
    /// The stdout_logfile value can contain Python string expressions that will evaluated against
    /// a dictionary that contains the keys
    /// `group_name`, `host_node_name`, `process_num`, `program_name`, and `here` (the directory of the taskmasterd config file).
    /// Default: `AUTO`
    /// Required: No
    stdout_logfile: Option<PathBuf>,

    /// The maximum number of bytes that may be consumed by stdout_logfile before it is rotated
    /// (suffix multipliers like “KB”, “MB”, and “GB” can be used in the value).
    /// Set this value to 0 to indicate an unlimited log size.
    /// Default: `50MB`
    /// Required: No
    stdout_logfile_maxbytes: Option<i32>,

    /// The number of stdout_logfile backups to keep around resulting from process stdout log file rotation.
    /// If set to 0, no backups will be kept.
    /// Default: `10`
    /// Required: No
    stdout_logfile_backups: Option<i32>,

    /// Max number of bytes written to capture FIFO when process is in “stdout capture mode” (see Capture Mode).
    /// Should be an integer (suffix multipliers like “KB”, “MB” and “GB” can used in the value).
    /// If this value is 0, process capture mode will be off.
    /// Default: `0`
    /// Required: No
    stdout_capture_maxbytes: Option<i32>,

    /// If `true`, PROCESS_LOG_STDOUT events will be emitted when the process writes to its stdout file descriptor.
    /// The events will only be emitted if the file descriptor is not in capture mode at the time the data is received (see Capture Mode).
    /// Default: `0`
    /// Required: No
    stdout_events_enabled: Option<bool>,

    /// If `true`, stdout will be directed to syslog along with the process name.
    /// Default: `False`
    /// Required: No
    stdout_syslog: Option<bool>,

    /// Put process stderr output in this file unless redirect_stderr is `true`.
    /// Accepts the same value types as stdout_logfile and may contain the same Python string expressions.
    /// Default: `AUTO`
    /// Required: No
    stderr_logfile: Option<PathBuf>,

    /// The maximum number of bytes before logfile rotation for stderr_logfile.
    /// Accepts the same value types as stdout_logfile_maxbytes.
    /// Default: `50MB`
    /// Required: No
    stderr_logfile_maxbytes: Option<i32>,

    /// The number of backups to keep around resulting from process stderr log file rotation. If set to `0`, no backups will be kept.
    /// Default: `10`
    /// Required: No
    stderr_logfile_backups: Option<i32>,

    /// Max number of bytes written to capture FIFO when process is in “stderr capture mode” (see Capture Mode).
    /// Should be an integer (suffix multipliers like “KB”, “MB” and “GB” can used in the value).
    /// If this value is `0`, process capture mode will be off.
    /// Default: `0`
    /// Required: No
    stderr_capture_maxbytes: Option<i32>,

    /// If `true`, PROCESS_LOG_STDERR events will be emitted when the process writes to its stderr file descriptor.
    /// The events will only be emitted if the file descriptor is not in capture mode at the time the data is received (see Capture Mode).
    /// Default: `false`
    /// Required: No
    stderr_events_enabled: Option<bool>,

    /// If `true`, stderr will be directed to syslog along with the process name.
    /// Default: `False`
    /// Required: No
    stderr_syslog: Option<bool>,

    /// A list of key/value pairs in the form KEY="val",KEY2="val2" that will be placed in the child process’ environment.
    /// The environment string may contain Python string expressions that will be evaluated against a dictionary
    /// containing `group_name`, `host_node_name`, `process_num`, `program_name`, and here (the directory of the taskmasterd config file).
    /// Values containing non-alphanumeric characters should be quoted (e.g. KEY="val:123",KEY2="val,456").
    /// Otherwise, quoting the values is optional but recommended.
    /// Note that the subprocess will inherit the environment variables of the shell used to start “taskmasterd” except for the
    /// ones overridden here.
    /// Default: No extra environment
    /// Required: No
    environment: Option<HashMap<String, String>>,

    /// A file path representing a directory to which taskmasterd should temporarily chdir before exec’ing the child.
    /// Default: No chdir (inherit supervisor’s)
    /// Required: No
    directory: Option<PathBuf>,

    /// An octal number (e.g. 002, 022) representing the umask of the process.
    /// Default: No special umask (inherit taskmaster’s)
    /// Required: No
    umask: Option<String>, // https://docs.rs/umask/1.0.0/umask/

    /// The URL passed in the environment to the subprocess process as TASKMASTER_SERVER_URL (see taskmaster.childutils)
    /// to allow the subprocess to easily communicate with the internal HTTP server.
    /// If provided, it should have the same syntax and structure as the [taskmasterctl] section option of the same name.
    /// If this is set to `AUTO`, or is unset, taskmaster will automatically construct a server URL, giving preference
    /// to a server that listens on UNIX domain sockets over one that listens on an internet socket.
    /// Default: `AUTO`
    /// Required: No
    serverurl: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum LogLevel {
    Critical,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Blather,
}

/// The Taskmasterd.conf file contains a section named [supervisord]
/// in which global settings related to the taskmasterd process should be inserted.
///
/// The logging level, dictating what is written to the supervisord activity log.
/// One of critical, error, warn, info, debug, trace, or blather.
///
/// Note that at log level debug, the supervisord log file will record the
/// stderr/stdout output of its child processes and extended info info about
/// process state changes, which is useful for debugging a process which isn’t starting properly.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Taskmasterd {
    /// The path to the activity log of the taskmasterd process.
    /// This option can include the value %(here)s, which expands
    /// to the directory in which the taskmasterd configuration file was found.
    /// Default: $CWD/supervisord.log
    /// Required: No
    logfile: Option<PathBuf>,

    /// The maximum number of bytes that may be consumed by the activity log file
    /// before it is rotated (suffix multipliers like “KB”, “MB”, and “GB” can be used in the value).
    /// Set this value to 0 to indicate an unlimited log size.
    /// Default: 50MB
    /// Required: No
    logfile_maxbytes: Option<i32>, // Should with define an other type for KB MB etc to be used ?

    /// The number of backups to keep around resulting from activity log file rotation.
    /// If set to 0, no backups will be kept.
    /// Default: 10
    /// Required: No
    logfile_backups: Option<i32>,

    /// The logging level, dictating what is written to the taskmasterd activity log.
    /// One of critical, error, warn, info, debug, trace, or blather.
    /// Note that at log level debug, the taskmasterd log file will record
    /// the stderr/stdout output of its child processes and extended info info about
    /// process state changes, which is useful for debugging a process which isn’t starting properly.
    /// Default: 'info'
    /// Required: No
    loglevel: Option<LogLevel>,

    /// The location in which taskmasterd keeps its pid file.
    /// This option can include the value %(here)s,
    /// which expands to the directory in which the taskmasterd configuration file was found.
    /// Default: $CWD/supervisord.pid
    /// Required: No
    pidfile: Option<i32>,

    /// The umask of the supervisord process.
    /// Default: 022
    /// Required: No
    umask: Option<String>,

    /// If true, supervisord will start in the foreground instead of daemonizing.
    /// Default: false
    /// Required: No
    nodaemon: Option<bool>,

    /// If true and not daemonized, logs will not be directed to stdout.
    /// Default: false
    /// Required: No
    silent: Option<bool>,

    /// The minimum number of file descriptors that must be available before
    /// taskmasterd will start successfully.
    /// A call to setrlimit will be made to attempt to raise the soft and hard limits
    /// of the taskmasterd process to satisfy minfds. The hard limit may only be raised
    /// if taskmasterd is run as root.
    /// Taskmasterd uses file descriptors liberally, and will enter a failure mode
    /// when one cannot be obtained from the OS, so it’s useful to be able to specify
    /// a minimum value to ensure it doesn’t run out of them during execution.
    /// These limits will be inherited by the managed subprocesses.
    /// This option is particularly useful on Solaris, which has a low per-process fd limit by default.
    /// Default: 1024
    /// Required: No
    minfds: Option<i32>,

    /// The minimum number of process descriptors that must be available before
    /// taskmasterd will start successfully.
    /// A call to setrlimit will be made to attempt to raise the soft and hard limits
    /// of the taskmasterd process to satisfy minprocs.
    /// The hard limit may only be raised if taskmasterd is run as root.
    /// taskmasterd will enter a failure mode when the OS runs out of process descriptors,
    /// so it’s useful to ensure that enough process descriptors are available upon taskmasterd startup.
    /// Default: 200
    /// Required: No
    minprocs: Option<i32>,

    /// Prevent taskmasterd from clearing any existing AUTO child log files at startup time.
    /// Useful for debugging.
    /// Default: false
    /// Required: No
    nocleanup: Option<bool>,

    /// The directory used for AUTO child log files.
    /// This option can include the value %(here)s, which expands to the directory
    /// in which the taskmasterd configuration file was found.
    /// Default: value of Python’s tempfile.gettempdir()
    /// Required: No
    childlogdir: PathBuf,

    /// Instruct taskmasterd to switch users to this UNIX user account before
    /// doing any meaningful processing. (e.g: antoine)
    /// The user can only be switched if taskmasterd is started as the root user.
    /// Default: do not switch users
    /// Required: No
    user: Option<String>,

    /// When taskmasterd daemonizes, switch to this directory.
    /// This option can include the value %(here)s, which expands
    /// to the directory in which the taskmasterd configuration file was found.
    /// Default: do not cd
    /// Required: No
    directory: Option<PathBuf>,

    /// Strip all ANSI escape sequences from child log files.
    /// Default: false
    /// Required: No
    strip_ansi: Option<bool>,

    /// A list of key/value pairs in the form KEY="val",KEY2="val2" that will be placed
    /// in the environment of all child processes.
    /// This does not change the environment of taskmasterd itself.
    /// This option can include the value %(here)s, which expands to the directory
    /// in which the taskmasterd configuration file was found.
    /// Values containing non-alphanumeric characters should be quoted (e.g. KEY="val:123",KEY2="val,456").
    /// Otherwise, quoting the values is optional but recommended.
    /// To escape percent characters, simply use two. (e.g. URI="/first%%20name")
    /// Note that subprocesses will inherit the environment variables of the shell
    /// used to start taskmasterd except for the ones overridden here and within the program’s environment option.
    /// Default: no values
    /// Required: No
    environment: Option<HashMap<String, String>>,

    /// The identifier string for this taskmaster process, used by the RPC interface.
    /// Default: supervisor
    /// Required: No
    identifier: Option<String>,
}

/// The configuration file may contain settings for the taskmasterctl interactive shell program.
/// These options are listed below.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Taskmasterctl {
    /// The URL that should be used to access the taskmasterd server, e.g. http://localhost:9001.
    /// For UNIX domain sockets, use unix:///absolute/path/to/file.sock
    /// Default: http://localhost:9001
    /// Required: No
    serverurl: Option<String>,

    /// The username to pass to the taskmasterd server for use in authentication.
    /// This should be same as username from the taskmasterd server configuration
    /// for the port or UNIX domain socket you’re attempting to access.
    /// Default: No username
    /// Required: No
    username: Option<String>,

    /// The password to pass to the taskmasterd server for use in authentication.
    /// This should be the cleartext version of password from the taskmasterd server configuration
    /// for the port or UNIX domain socket you’re attempting to access.
    /// This value cannot be passed as a SHA hash.
    /// Unlike other passwords specified in this file, it must be provided in cleartext.
    /// Default: No password
    /// Required: No
    password: Option<String>,

    /// String used as taskmasterctl prompt.
    /// Default: taskmaster
    /// Required: No
    prompt: Option<String>,

    /// A path to use as the readline persistent history file.
    /// If you enable this feature by choosing a path, your taskmasterctl
    /// commands will be kept in the file, and you can use readline
    /// (e.g. arrow-up) to invoke commands you performed in your
    /// last taskmasterctl session.
    /// Default: No file
    /// Required: No
    history_file: Option<PathBuf>,
}

/// The taskmasterd.conf file contains a section named [unix_http_server]
/// under which configuration parameters for an HTTP server that listens
/// on a UNIX domain socket should be inserted.
/// If the configuration file has no [unix_http_server] section,
/// a UNIX domain socket HTTP server will not be started.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct UnixHttpServer {
    /// A path to a UNIX domain socket on which taskmaster will listen
    /// Default: None
    /// Required: No
    file: Option<String>,

    /// Change the UNIX permission mode bits of the UNIX domain socket to this value at startup
    /// Default: 0700
    /// Required: No
    chmod: Option<String>,

    /// Change the user and group of the socket file to this value.
    /// May be only a UNIX username (e.g. antoine)
    /// or a UNIX username + a group separated by a colon (e.g. antoine:wheel).
    /// Default: Use the username and group of the user who starts taskmasterd.
    /// Required: No
    chown: Option<String>,

    /// The username required for authentication to this HTTP server.
    /// Default: No username required
    /// Required: No
    username: Option<String>,

    /// The password required for authentication to this HTTP server.
    /// This can be a cleartext password, or can be specified as a SHA-1 hash if prefixed by the string {SHA}.
    /// For example, {SHA}82ab876d1387bfafe46cc1c8a2ef074eae50cb1d is the SHA-stored version of the password “thepassword”.
    /// Note that hashed password must be in hex format.
    /// Default: No password required
    /// Required: No
    password: Option<String>,
}

/// The taskmasterd.conf file contains a section named [inet_http_server]
/// under which configuration parameters for an HTTP server that listens
/// on a TCP (internet) socket should be inserted.
/// If the configuration file has no [inet_http_server] section,
/// an inet HTTP server will not be started.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct InetHttpServer {
    /// A TCP host:port value or (e.g. 127.0.0.1:9001) on which taskmaster will listen for HTTP/XML-RPC requests.
    /// taskmasterctl will use XML-RPC to communicate with taskmasterd over this port.
    /// To listen on all interfaces in the machine, use :9001 or *:9001.
    /// Default: No default
    /// Required: Yes
    /// This should also support IPV6 !
    port: String,

    /// The username required for authentication to this HTTP server.
    /// Default: No username required
    /// Required: No
    username: Option<String>,

    /// The password required for authentication to this HTTP server.
    /// This can be a cleartext password, or can be specified as a SHA-1 hash if prefixed by the string {SHA}.
    /// For example, {SHA}82ab876d1387bfafe46cc1c8a2ef074eae50cb1d is the SHA-stored version of the password “thepassword”.
    /// Note that hashed password must be in hex format.
    /// Default: No password required
    /// Required: No
    password: Option<String>,
}

/// Files replace the order and values of LOOKAT
/// The taskmaster.confi/yaml file may contain a section named [include].
/// If the configuration file contains an [include] section, it must contain a single key named “files”.
/// The values in this key specify other configuration files to be included within the configuration.
/// For example, supervisord.conf could be included when migrating from Supervisor to Taskmaster tools.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Include {
    /// A space-separated sequence of file globs.
    /// Each file glob may be absolute or relative.
    /// If the file glob is relative, it is considered relative to the location of the configuration file
    /// which includes it.
    /// A “glob” is a file pattern which matches a specified pattern according to the rules used by the Unix shell.
    /// No tilde expansion is done, but *, ?, and character ranges expressed with [] will be correctly matched.
    /// The string expression is evaluated against a dictionary that includes `host_node_name` and `here`
    /// (the directory of the taskmasterd config file).
    /// Recursive includes from included files are not supported.
    /// Default: No default (required)
    /// Required: Yes
    files: Vec<String>,
}

/// http://supervisord.org/configuration.html#group-x-section-settings
/// To place programs into a group so you can treat them as a unit, define a [group:x] section in your configuration file.
/// The group header value is a composite.
/// It is the word “group”, followed directly by a colon, then the group name.
/// A header value of [group:foo] describes a group with the name of “foo”.
/// The name is used within client applications that control the processes that are created as a result of this configuration.
/// It is an error to create a group section that does not have a name.
/// The name must not include a colon character or a bracket character.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// A comma-separated list of program names. The programs which are listed become members of the group.
    /// Default: No default (required)
    /// Required: Yes
    programs: String,

    /// A priority number analogous to a [program:x] priority value assigned to the group.
    /// Default: `999`
    /// Required: No
    priority: Option<i32>,
}

/// http://supervisord.org/configuration.html#fcgi-program-x-section-settings
/// Taskamaster can manage groups of FastCGI processes that all listen on the same socket.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct FcgiProgram {
    /// The FastCGI socket for this program, either TCP or UNIX domain socket.
    /// For TCP sockets, use this format: tcp://localhost:9002.
    /// For UNIX domain sockets, use unix:///absolute/path/to/file.sock.
    /// String expressions are evaluated against a dictionary containing the keys “program_name” and “here”
    /// (the directory of the taskmasterd config file).
    /// Default: No default
    /// Required: `Yes`
    socket: String, //=unix:///var/run/supervisor/%(program_name)s.sock

    /// Sets socket listen(2) backlog.
    /// Default: `socket.SOMAXCONN`
    /// Required: No
    socket_backlog: Option<String>,

    /// For UNIX domain sockets, this parameter can be used to specify the user and group for the FastCGI socket.
    /// May be a UNIX username (e.g. antoine) or a UNIX username and group separated by a colon (e.g. antoine:wheel).
    /// Default: Uses the user and group set for the fcgi-program
    /// Required: No
    socket_owner: Option<String>,

    /// For UNIX domain sockets, this parameter can be used to specify the permission mode.
    /// Default: `0700`
    /// Required: No
    socket_mode: Option<String>,

    /// The command that will be run when this program is started.
    /// The command can be either absolute (e.g. /path/to/programname) or relative (e.g. programname).
    /// If it is relative, the taskmasterd’s environment $PATH will be searched for the executable.
    /// Programs can accept arguments, e.g. /path/to/program foo bar.
    /// The command line can use double quotes to group arguments with spaces in them to pass to the program,
    /// e.g. /path/to/program/name -p "foo bar".
    /// Note that the value of command may include Python string expressions,
    /// e.g. /path/to/programname --port=80%(process_num)02d might expand to /path/to/programname --port=8000 at runtime.
    /// String expressions are evaluated against a dictionary containing the keys
    /// group_name, host_node_name, program_name, process_num, numprocs,
    /// here (the directory of the taskmasterd config file), and all taskmasterd’s environment variables prefixed
    /// with ENV_. Controlled programs should themselves not be daemons,
    /// as taskmasterd assumes it is responsible for daemonizing its subprocesses (see Nondaemonizing of Subprocesses).
    /// Default: No default
    /// Required: Yes
    command: String, // (PathBuf, String, ...)

    /// A Python string expression that is used to compose the taskmaster process name for this process.
    /// You usually don’t need to worry about setting this unless you change numprocs.
    /// The string expression is evaluated against a dictionary that includes
    /// group_name, host_node_name, process_num, program_name, and here (the directory of the taskmasterd config file).
    /// Default: %(program_name)s
    /// Required: No
    process_name: Option<String>,

    /// Taskmaster will start as many instances of this program as named by numprocs.
    /// Note that if numprocs > 1, the process_name expression must include %(process_num)s
    /// (or any other valid Python string expression that includes process_num) within it.
    /// Default: 1
    /// Required: No
    numprocs: Option<u16>,

    /// An integer offset that is used to compute the number at which numprocs starts.
    /// Default: 0
    /// Required: No
    numprocs_start: Option<u16>,

    /// The relative priority of the program in the start and shutdown ordering.
    /// Lower priorities indicate programs that start first and shut down last at
    /// startup and when aggregate commands are used in various clients
    /// (e.g. “start all”/”stop all”).
    /// Higher priorities indicate programs that start last and shut down first.
    /// Default: `999`
    /// Required: No
    priority: Option<i32>,

    /// If true, this program will start automatically when taskmasterd is started.
    /// Default: `true`
    /// Required: No
    autostart: Option<bool>,

    /// The total number of seconds which the program needs to stay running after a startup
    /// to consider the start successful (moving the process from the STARTING state to the RUNNING state).
    /// Set to 0 to indicate that the program needn’t stay running for any particular amount of time.
    /// Default: `1`
    /// Required: No
    startsecs: Option<i32>,

    /// The number of serial failure attempts that taskmasterd will allow when attempting to start
    /// the program before giving up and putting the process into an FATAL state.
    /// Default: `3`
    /// Required: No
    startretries: Option<i32>,

    /// Specifies if taskmasterd should automatically restart a process if it exits when it is
    /// in the RUNNING state.
    /// May be one of `false`, `unexpected`, or `true`.
    /// If `false`, the process will not be autorestarted.
    /// If `unexpected`, the process will be restarted when the program exits with an
    /// exit code that is not one of the exit codes associated with this process’ configuration.
    /// If `true`, the process will be unconditionally restarted when it exits, without regard to its exit code.
    /// Default: `unexpected`
    /// Required: No.
    autorestart: Option<Restart>,

    /// The list of “expected” exit codes for this program used with autorestart.
    /// If the autorestart parameter is set to `unexpected`, and the process exits
    /// in any other way than as a result of a tadskmaster stop request,
    /// taskmasterd will restart the process if it exits with an exit code that is not defined in this list.
    /// Default: `0`
    /// Required: No
    exitcodes: Option<Vec<i32>>,

    /// The signal used to kill the program when a stop is requested.
    /// This can be any of `TERM`, `HUP`, `INT`, `QUIT`, `KILL`, `USR1`, or `USR2`.
    /// Default: `TERM`
    /// Required: No
    stopsignal: Option<Vec<String>>,

    /// The number of seconds to wait for the OS to return a `SIGCHLD` to taskmasterd after the program
    /// has been sent a stopsignal.
    /// If this number of seconds elapses before taskmasterd receives a SIGCHLD from the process,
    /// taskmasterd will attempt to kill it with a final `SIGKILL`.
    /// Default: `10`
    /// Required: No
    stopwaitsecs: Option<i32>,

    /// If `true`, the flag causes taskmaster to send the stop signal to the whole process group
    /// and implies killasgroup is `true`.
    /// This is useful for programs, such as Flask in debug mode, that do not propagate
    /// stop signals to their children, leaving them orphaned.
    /// Default: `false`
    /// Required: No
    stopasgroup: Option<bool>,

    /// If true, when resorting to send `SIGKILL` to the program to terminate it send it to its
    /// whole process group instead, taking care of its children as well,
    /// useful e.g with Python programs using multiprocessing.
    /// Default: `false`
    /// Required: No
    killasgroup: Option<bool>,

    /// Instruct taskmasterd to use this UNIX user account as the account which runs the program.
    /// The user can only be switched if taskmasterd is run as the root user.
    /// If taskmasterd can’t switch to the specified user, the program will not be started.
    /// Default: Do not switch users
    /// Required: No
    user: Option<String>,

    /// If `true`, cause the process’ stderr output to be sent back to taskmasterd on its
    /// stdout file descriptor
    /// (in UNIX shell terms, this is the equivalent of executing /the/program 2>&1).
    /// Default: `false`
    /// Required: No
    redirect_stderr: Option<bool>,

    /// Put process stdout output in this file (and if redirect_stderr is true, also place
    /// stderr output in this file).
    /// If stdout_logfile is unset or set to `AUTO`, taskmaster will automatically choose a file location.
    /// If this is set to `NONE`, taskmasterd will create no log file.
    /// `AUTO` log files and their backups will be deleted when taskmasterd restarts.
    /// The stdout_logfile value can contain Python string expressions that will evaluated against
    /// a dictionary that contains the keys
    /// `group_name`, `host_node_name`, `process_num`, `program_name`, and `here` (the directory of the taskmasterd config file).
    /// Default: `AUTO`
    /// Required: No
    stdout_logfile: Option<PathBuf>,

    /// The maximum number of bytes that may be consumed by stdout_logfile before it is rotated
    /// (suffix multipliers like “KB”, “MB”, and “GB” can be used in the value).
    /// Set this value to 0 to indicate an unlimited log size.
    /// Default: `50MB`
    /// Required: No
    stdout_logfile_maxbytes: Option<i32>,

    /// The number of stdout_logfile backups to keep around resulting from process stdout log file rotation.
    /// If set to 0, no backups will be kept.
    /// Default: `10`
    /// Required: No
    stdout_logfile_backups: Option<i32>,

    /// Max number of bytes written to capture FIFO when process is in “stdout capture mode” (see Capture Mode).
    /// Should be an integer (suffix multipliers like “KB”, “MB” and “GB” can used in the value).
    /// If this value is 0, process capture mode will be off.
    /// Default: `0`
    /// Required: No
    stdout_capture_maxbytes: Option<i32>,

    /// If `true`, PROCESS_LOG_STDOUT events will be emitted when the process writes to its stdout file descriptor.
    /// The events will only be emitted if the file descriptor is not in capture mode at the time the data is received (see Capture Mode).
    /// Default: `0`
    /// Required: No
    stdout_events_enabled: Option<bool>,

    /// If `true`, stdout will be directed to syslog along with the process name.
    /// Default: `False`
    /// Required: No
    stdout_syslog: Option<bool>,

    /// Put process stderr output in this file unless redirect_stderr is `true`.
    /// Accepts the same value types as stdout_logfile and may contain the same Python string expressions.
    /// Default: `AUTO`
    /// Required: No
    stderr_logfile: Option<PathBuf>,

    /// The maximum number of bytes before logfile rotation for stderr_logfile.
    /// Accepts the same value types as stdout_logfile_maxbytes.
    /// Default: `50MB`
    /// Required: No
    stderr_logfile_maxbytes: Option<i32>,

    /// The number of backups to keep around resulting from process stderr log file rotation. If set to `0`, no backups will be kept.
    /// Default: `10`
    /// Required: No
    stderr_logfile_backups: Option<i32>,

    /// Max number of bytes written to capture FIFO when process is in “stderr capture mode” (see Capture Mode).
    /// Should be an integer (suffix multipliers like “KB”, “MB” and “GB” can used in the value).
    /// If this value is `0`, process capture mode will be off.
    /// Default: `0`
    /// Required: No
    stderr_capture_maxbytes: Option<i32>,

    /// If `true`, PROCESS_LOG_STDERR events will be emitted when the process writes to its stderr file descriptor.
    /// The events will only be emitted if the file descriptor is not in capture mode at the time the data is received (see Capture Mode).
    /// Default: `false`
    /// Required: No
    stderr_events_enabled: Option<bool>,

    /// If `true`, stderr will be directed to syslog along with the process name.
    /// Default: `False`
    /// Required: No
    stderr_syslog: Option<bool>,

    /// A list of key/value pairs in the form KEY="val",KEY2="val2" that will be placed in the child process’ environment.
    /// The environment string may contain Python string expressions that will be evaluated against a dictionary
    /// containing `group_name`, `host_node_name`, `process_num`, `program_name`, and here (the directory of the taskmasterd config file).
    /// Values containing non-alphanumeric characters should be quoted (e.g. KEY="val:123",KEY2="val,456").
    /// Otherwise, quoting the values is optional but recommended.
    /// Note that the subprocess will inherit the environment variables of the shell used to start “taskmasterd” except for the
    /// ones overridden here.
    /// Default: No extra environment
    /// Required: No
    environment: Option<HashMap<String, String>>,

    /// A file path representing a directory to which taskmasterd should temporarily chdir before exec’ing the child.
    /// Default: No chdir (inherit supervisor’s)
    /// Required: No
    directory: Option<PathBuf>,

    /// An octal number (e.g. 002, 022) representing the umask of the process.
    /// Default: No special umask (inherit taskmaster’s)
    /// Required: No
    umask: Option<String>, // https://docs.rs/umask/1.0.0/umask/

    /// The URL passed in the environment to the subprocess process as TASKMASTER_SERVER_URL (see taskmaster.childutils)
    /// to allow the subprocess to easily communicate with the internal HTTP server.
    /// If provided, it should have the same syntax and structure as the [taskmasterctl] section option of the same name.
    /// If this is set to `AUTO`, or is unset, taskmaster will automatically construct a server URL, giving preference
    /// to a server that listens on UNIX domain sockets over one that listens on an internet socket.
    /// Default: `AUTO`
    /// Required: No
    serverurl: Option<String>,
}

/// Taskmaster allows specialized homogeneous process groups (“event listener pools”) to be defined
/// within the configuration file.
/// These pools contain processes that are meant to receive and respond to event notifications
/// from taskmaster’s event system.
/// http://supervisord.org/events.html#event-types
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct EventListener {
    /// The event listener pool’s event queue buffer size. When a listener pool’s event buffer
    /// is overflowed (as can happen when an event listener pool cannot keep up with all of the
    /// events sent to it),the oldest event in the buffer is discarded.
    /// Default: `10`
    /// Required: No
    buffer_size: Option<i32>,

    /// Event types that may be subscribed to by event listeners are predefined by taskmaster
    /// and fall into several major categories, including “process state change”, “process communication”,
    /// and “supervisor state change” events.
    /// Default: No default value
    /// Required: No
    events: Option<HashMap<String, String>>,

    /// A pkg_resources entry point string that resolves to a Python callable.
    /// The default value is taskmaster.dispatchers:default_handler.
    /// Specifying an alternate result handler is a very uncommon thing to need to do,
    /// and as a result, how to create one is not documented.
    /// Default: `??`
    /// Required: No
    result_handler: Option<String>,

    /// The command that will be run when this program is started.
    /// The command can be either absolute (e.g. /path/to/programname) or relative (e.g. programname).
    /// If it is relative, the taskmasterd’s environment $PATH will be searched for the executable.
    /// Programs can accept arguments, e.g. /path/to/program foo bar.
    /// The command line can use double quotes to group arguments with spaces in them to pass to the program,
    /// e.g. /path/to/program/name -p "foo bar".
    /// Note that the value of command may include Python string expressions,
    /// e.g. /path/to/programname --port=80%(process_num)02d might expand to /path/to/programname --port=8000 at runtime.
    /// String expressions are evaluated against a dictionary containing the keys
    /// group_name, host_node_name, program_name, process_num, numprocs,
    /// here (the directory of the taskmasterd config file), and all taskmasterd’s environment variables prefixed
    /// with ENV_. Controlled programs should themselves not be daemons,
    /// as taskmasterd assumes it is responsible for daemonizing its subprocesses (see Nondaemonizing of Subprocesses).
    /// Default: No default
    /// Required: Yes
    command: String, // (PathBuf, String, ...)

    /// A Python string expression that is used to compose the taskmaster process name for this process.
    /// You usually don’t need to worry about setting this unless you change numprocs.
    /// The string expression is evaluated against a dictionary that includes
    /// group_name, host_node_name, process_num, program_name, and here (the directory of the taskmasterd config file).
    /// Default: %(program_name)s
    /// Required: No
    process_name: Option<String>,

    /// Taskmaster will start as many instances of this program as named by numprocs.
    /// Note that if numprocs > 1, the process_name expression must include %(process_num)s
    /// (or any other valid Python string expression that includes process_num) within it.
    /// Default: 1
    /// Required: No
    numprocs: Option<u16>,

    /// An integer offset that is used to compute the number at which numprocs starts.
    /// Default: 0
    /// Required: No
    numprocs_start: Option<u16>,

    /// The relative priority of the program in the start and shutdown ordering.
    /// Lower priorities indicate programs that start first and shut down last at
    /// startup and when aggregate commands are used in various clients
    /// (e.g. “start all”/”stop all”).
    /// Higher priorities indicate programs that start last and shut down first.
    /// Default: `999`
    /// Required: No
    priority: Option<i32>,

    /// If true, this program will start automatically when taskmasterd is started.
    /// Default: `true`
    /// Required: No
    autostart: Option<bool>,

    /// The total number of seconds which the program needs to stay running after a startup
    /// to consider the start successful (moving the process from the STARTING state to the RUNNING state).
    /// Set to 0 to indicate that the program needn’t stay running for any particular amount of time.
    /// Default: `1`
    /// Required: No
    startsecs: Option<i32>,

    /// The number of serial failure attempts that taskmasterd will allow when attempting to start
    /// the program before giving up and putting the process into an FATAL state.
    /// Default: `3`
    /// Required: No
    startretries: Option<i32>,

    /// Specifies if taskmasterd should automatically restart a process if it exits when it is
    /// in the RUNNING state.
    /// May be one of `false`, `unexpected`, or `true`.
    /// If `false`, the process will not be autorestarted.
    /// If `unexpected`, the process will be restarted when the program exits with an
    /// exit code that is not one of the exit codes associated with this process’ configuration.
    /// If `true`, the process will be unconditionally restarted when it exits, without regard to its exit code.
    /// Default: `unexpected`
    /// Required: No.
    autorestart: Option<Restart>,

    /// The list of “expected” exit codes for this program used with autorestart.
    /// If the autorestart parameter is set to `unexpected`, and the process exits
    /// in any other way than as a result of a tadskmaster stop request,
    /// taskmasterd will restart the process if it exits with an exit code that is not defined in this list.
    /// Default: `0`
    /// Required: No
    exitcodes: Option<Vec<i32>>,

    /// The signal used to kill the program when a stop is requested.
    /// This can be any of `TERM`, `HUP`, `INT`, `QUIT`, `KILL`, `USR1`, or `USR2`.
    /// Default: `TERM`
    /// Required: No
    stopsignal: Option<Vec<String>>,

    /// The number of seconds to wait for the OS to return a `SIGCHLD` to taskmasterd after the program
    /// has been sent a stopsignal.
    /// If this number of seconds elapses before taskmasterd receives a SIGCHLD from the process,
    /// taskmasterd will attempt to kill it with a final `SIGKILL`.
    /// Default: `10`
    /// Required: No
    stopwaitsecs: Option<i32>,

    /// If `true`, the flag causes taskmaster to send the stop signal to the whole process group
    /// and implies killasgroup is `true`.
    /// This is useful for programs, such as Flask in debug mode, that do not propagate
    /// stop signals to their children, leaving them orphaned.
    /// Default: `false`
    /// Required: No
    stopasgroup: Option<bool>,

    /// If true, when resorting to send `SIGKILL` to the program to terminate it send it to its
    /// whole process group instead, taking care of its children as well,
    /// useful e.g with Python programs using multiprocessing.
    /// Default: `false`
    /// Required: No
    killasgroup: Option<bool>,

    /// Instruct taskmasterd to use this UNIX user account as the account which runs the program.
    /// The user can only be switched if taskmasterd is run as the root user.
    /// If taskmasterd can’t switch to the specified user, the program will not be started.
    /// Default: Do not switch users
    /// Required: No
    user: Option<String>,

    /// If `true`, cause the process’ stderr output to be sent back to taskmasterd on its
    /// stdout file descriptor
    /// (in UNIX shell terms, this is the equivalent of executing /the/program 2>&1).
    /// Default: `false`
    /// Required: No
    redirect_stderr: Option<bool>,

    /// Put process stdout output in this file (and if redirect_stderr is true, also place
    /// stderr output in this file).
    /// If stdout_logfile is unset or set to `AUTO`, taskmaster will automatically choose a file location.
    /// If this is set to `NONE`, taskmasterd will create no log file.
    /// `AUTO` log files and their backups will be deleted when taskmasterd restarts.
    /// The stdout_logfile value can contain Python string expressions that will evaluated against
    /// a dictionary that contains the keys
    /// `group_name`, `host_node_name`, `process_num`, `program_name`, and `here` (the directory of the taskmasterd config file).
    /// Default: `AUTO`
    /// Required: No
    stdout_logfile: Option<PathBuf>,

    /// The maximum number of bytes that may be consumed by stdout_logfile before it is rotated
    /// (suffix multipliers like “KB”, “MB”, and “GB” can be used in the value).
    /// Set this value to 0 to indicate an unlimited log size.
    /// Default: `50MB`
    /// Required: No
    stdout_logfile_maxbytes: Option<i32>,

    /// The number of stdout_logfile backups to keep around resulting from process stdout log file rotation.
    /// If set to 0, no backups will be kept.
    /// Default: `10`
    /// Required: No
    stdout_logfile_backups: Option<i32>,

    /// Max number of bytes written to capture FIFO when process is in “stdout capture mode” (see Capture Mode).
    /// Should be an integer (suffix multipliers like “KB”, “MB” and “GB” can used in the value).
    /// If this value is 0, process capture mode will be off.
    /// Default: `0`
    /// Required: No
    stdout_capture_maxbytes: Option<i32>,

    /// If `true`, PROCESS_LOG_STDOUT events will be emitted when the process writes to its stdout file descriptor.
    /// The events will only be emitted if the file descriptor is not in capture mode at the time the data is received (see Capture Mode).
    /// Default: `0`
    /// Required: No
    stdout_events_enabled: Option<bool>,

    /// If `true`, stdout will be directed to syslog along with the process name.
    /// Default: `False`
    /// Required: No
    stdout_syslog: Option<bool>,

    /// Put process stderr output in this file unless redirect_stderr is `true`.
    /// Accepts the same value types as stdout_logfile and may contain the same Python string expressions.
    /// Default: `AUTO`
    /// Required: No
    stderr_logfile: Option<PathBuf>,

    /// The maximum number of bytes before logfile rotation for stderr_logfile.
    /// Accepts the same value types as stdout_logfile_maxbytes.
    /// Default: `50MB`
    /// Required: No
    stderr_logfile_maxbytes: Option<i32>,

    /// The number of backups to keep around resulting from process stderr log file rotation. If set to `0`, no backups will be kept.
    /// Default: `10`
    /// Required: No
    stderr_logfile_backups: Option<i32>,

    /// Max number of bytes written to capture FIFO when process is in “stderr capture mode” (see Capture Mode).
    /// Should be an integer (suffix multipliers like “KB”, “MB” and “GB” can used in the value).
    /// If this value is `0`, process capture mode will be off.
    /// Default: `0`
    /// Required: No
    stderr_capture_maxbytes: Option<i32>,

    /// If `true`, PROCESS_LOG_STDERR events will be emitted when the process writes to its stderr file descriptor.
    /// The events will only be emitted if the file descriptor is not in capture mode at the time the data is received (see Capture Mode).
    /// Default: `false`
    /// Required: No
    stderr_events_enabled: Option<bool>,

    /// If `true`, stderr will be directed to syslog along with the process name.
    /// Default: `False`
    /// Required: No
    stderr_syslog: Option<bool>,

    /// A list of key/value pairs in the form KEY="val",KEY2="val2" that will be placed in the child process’ environment.
    /// The environment string may contain Python string expressions that will be evaluated against a dictionary
    /// containing `group_name`, `host_node_name`, `process_num`, `program_name`, and here (the directory of the taskmasterd config file).
    /// Values containing non-alphanumeric characters should be quoted (e.g. KEY="val:123",KEY2="val,456").
    /// Otherwise, quoting the values is optional but recommended.
    /// Note that the subprocess will inherit the environment variables of the shell used to start “taskmasterd” except for the
    /// ones overridden here.
    /// Default: No extra environment
    /// Required: No
    environment: Option<HashMap<String, String>>,

    /// A file path representing a directory to which taskmasterd should temporarily chdir before exec’ing the child.
    /// Default: No chdir (inherit supervisor’s)
    /// Required: No
    directory: Option<PathBuf>,

    /// An octal number (e.g. 002, 022) representing the umask of the process.
    /// Default: No special umask (inherit taskmaster’s)
    /// Required: No
    umask: Option<String>, // https://docs.rs/umask/1.0.0/umask/

    /// The URL passed in the environment to the subprocess process as TASKMASTER_SERVER_URL (see taskmaster.childutils)
    /// to allow the subprocess to easily communicate with the internal HTTP server.
    /// If provided, it should have the same syntax and structure as the [taskmasterctl] section option of the same name.
    /// If this is set to `AUTO`, or is unset, taskmaster will automatically construct a server URL, giving preference
    /// to a server that listens on UNIX domain sockets over one that listens on an internet socket.
    /// Default: `AUTO`
    /// Required: No
    serverurl: Option<String>,
}

/// Adding rpcinterface:x settings in the configuration file is only useful for people who wish
/// to extend taskmaster with additional custom behavior.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct RpcInterface {
    /// pkg_resources “entry point” dotted name to your RPC interface’s factory function.
    /// Default: N/A
    /// Required: No
    rpcinterface_factory: Option<String>, // taskmaster.rpcinterface:make_main_rpcinterface

    /// Default: `1`
    /// Required: No
    retries: Option<i32>,
}

/// File order it will look at, and pick the first it found.
mod config {
    use super::*;

    /// LOOKAT is Default values of Include::files
    /// It contains path to taskmasterd configuration files.
    /// Path can be customized including `include` section.
    const LOOKAT: [&'static str; 6] = [
        "../etc/taskmasterd.yaml",
        "../taskmasterd.yaml",
        "./taskmasterd.yaml",
        "./etc/taskmasterd.yaml",
        "/etc/taskmasterdd.yaml",
        "/etc/taskmaster/taskmasterd.conf",
    ];

    /// Returns the first found configuration file following order in LOOKAT
    /// of include if specified.
    pub fn find_file() -> Result<&'static &'static str, Box<dyn Error>> {
        match LOOKAT.iter().find(|path| Path::new(path).exists()) {
            Some(p) => return Ok(p),
            None => return Err("Could not find any configuration file.".into()),
        };
    }
}

impl Config {
    pub fn parse(filename: Option<String>) -> Result<Config, Box<dyn Error>> {
        let file = match filename {
            Some(f) => File::open(&f)?,
            None => {
                let valid_path_to_conf = config::find_file()?;
                File::open(&valid_path_to_conf)?
            }
        };
        let deserialized_conf: Config = serde_yaml::from_reader(file)?;
        Ok(deserialized_conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_one_program() {
        let filename = Some(String::from("./config_files/one_program.yaml"));
        let deser = Config::parse(filename).unwrap();
        let one_program = Config {
            programs: Some({
                (0..1)
                    .map(|_| {
                        (
                            String::from("ls"),
                            Program {
                                command: String::from("/bin/ls -l"),
                                process_name: None,
                                numprocs: None,
                                numprocs_start: None,
                                priority: None,
                                autostart: None,
                                startsecs: None,
                                startretries: None,
                                autorestart: None,
                                exitcodes: None,
                                stopsignal: None,
                                stopwaitsecs: None,
                                stopasgroup: None,
                                killasgroup: None,
                                user: None,
                                redirect_stderr: None,
                                stdout_logfile: None,
                                stdout_logfile_maxbytes: None,
                                stdout_logfile_backups: None,
                                stdout_capture_maxbytes: None,
                                stdout_events_enabled: None,
                                stdout_syslog: None,
                                stderr_logfile: None,
                                stderr_logfile_maxbytes: None,
                                stderr_logfile_backups: None,
                                stderr_capture_maxbytes: None,
                                stderr_events_enabled: None,
                                stderr_syslog: None,
                                environment: None,
                                directory: None,
                                umask: None,
                                serverurl: None,
                            },
                        )
                    })
                    .collect()
            }),
            taskmasterd: None,
            taskmasterctl: None,
            unix_http_server: None,
            inet_http_server: None,
            include: None,
            group: None,
            fcgi_program: None,
            eventlistener: None,
            rpcinterface: None,
        };
        assert_eq!(deser, one_program);
    }
}
