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
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Restart {
    Never,
    Always,
    Unexpected,
}

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Program {
    command: String,
    process_name: Option<String>,
    numprocs: Option<u16>,
    directory: Option<PathBuf>,
    umask: Option<String>, // https://docs.rs/umask/1.0.0/umask/
    priority: Option<i32>,
    autostart: Option<bool>,
    autorestart: Option<Restart>,
    startsecs: Option<i32>,
    startretries: Option<i32>,
    exitcodes: Option<Vec<i32>>,
    stopsignal: Option<Vec<String>>,
    stopwaitsecs: Option<i32>,
    stopasgroup: Option<bool>,
    killasgroup: Option<bool>,
    user: Option<String>,
    redirect_stderr: Option<bool>,
    stdout_logfile: Option<PathBuf>,
    stdout_logfile_maxbytes: Option<i32>,
    stdout_logfile_backups: Option<i32>,
    stdout_capture_maxbytes: Option<i32>,
    stdout_events_enabled: Option<bool>,
    stderr_logfile: Option<PathBuf>,
    stderr_logfile_maxbytes: Option<i32>,
    stderr_logfile_backups: Option<i32>,
    stderr_capture_maxbytes: Option<i32>,
    stderr_events_enabled: Option<bool>,
    environment: Option<HashMap<String, String>>,
    serverurl: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Taskmasterctl {
    serverurl: String,
    username: String,
    password: String,
    prompt: String,
}


/// The taskmasterd.conf file contains a section named [unix_http_server]
/// under which configuration parameters for an HTTP server that listens
/// on a UNIX domain socket should be inserted.
/// If the configuration file has no [unix_http_server] section,
/// a UNIX domain socket HTTP server will not be started.
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Include {
    files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Group {
    programs: String,
    priority: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FcgiProgram {
    command: String,
    socket: String, //=unix:///var/run/supervisor/%(program_name)s.sock
    socket_owner: String,
    socket_mode: String,
    process_name: String,
    numprocs: u16,
    directory: PathBuf,
    umask: String, // https://docs.rs/umask/1.0.0/umask/
    priority: i32,
    autostart: bool,
    autorestart: Restart,
    startsecs: i32,
    startretries: i32,
    exitcodes: Vec<i32>,
    stopsignal: Vec<String>,
    stopwaitsecs: i32,
    stopasgroup: bool,
    killasgroup: bool,
    user: String,
    redirect_stderr: bool,
    stdout_logfile: PathBuf,
    stdout_logfile_maxbytes: i32,
    stdout_logfile_backups: i32,
    stdout_events_enabled: bool,
    stderr_logfile: PathBuf,
    stderr_logfile_maxbytes: i32,
    stderr_logfile_backups: i32,
    stderr_events_enabled: bool,
    environment: HashMap<String, String>,
    serverurl: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EventListener {
    command: String,
    process_name: String,
    numprocs: u16,
    events: String, //PROCESS_STATE
    buffer_size: i32,
    directory: PathBuf,
    umask: String, // https://docs.rs/umask/1.0.0/umask/
    priority: i32,
    autostart: bool,
    autorestart: Restart,
    startsecs: i32,
    startretries: i32,
    exitcodes: Vec<i32>,
    stopsignal: Vec<String>,
    stopwaitsecs: i32,
    stopasgroup: bool,
    killasgroup: bool,
    user: String,
    redirect_stderr: bool,
    stdout_logfile: PathBuf,
    stdout_logfile_maxbytes: i32,
    stdout_logfile_backups: i32,
    stdout_events_enabled: bool,
    stderr_logfile: PathBuf,
    stderr_logfile_maxbytes: i32,
    stderr_logfile_backups: i32,
    stderr_events_enabled: bool,
    environment: HashMap<String, String>,
    serverurl: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RpcInterface {
    rpcinterface_factory: String, // taskmaster.rpcinterface:make_main_rpcinterface
    retries: i32,
}

/// File order it will look at, and pick the first it found.
mod config {
    use super::*;

    // LOOKAT is Default values of Include::files
    const LOOKAT: [&'static str; 6] = [
        "../etc/taskmasterd.yaml",
        "../taskmasterd.yaml",
        "./taskmasterd.yaml",
        "./etc/taskmasterd.yaml",
        "/etc/taskmasterdd.yaml",
        "/etc/taskmaster/taskmasterd.conf",
    ];

    pub fn find_file() -> Result<&'static &'static str, Box<dyn Error>> {
        match LOOKAT.iter().find(|path| Path::new(path).exists()) {
            Some(p) => return Ok(p),
            None => return Err("Could not find any configuration file.".into()),
        };
    }
}

impl Config {
    pub fn parse() -> Result<Config, Box<dyn Error>> {
        let valid_path_to_conf = config::find_file()?;
        let file = File::open(&valid_path_to_conf)?;
        let deserialized_conf: Config = serde_yaml::from_reader(file)?;
        Ok(deserialized_conf)
    }
}

#[cfg(test)]
mod tests {
    //    use super::*;

    #[test]
    fn empty_default_values() {}
}
