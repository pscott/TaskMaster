//! # Config
//!
//! Library parsing the taskmasterd and taskmasterctl configuration files.
//! Ref http://supervisord.org/configuration.html#unix-http-server-section-settings 

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryFrom,
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
#[serde(rename_all = "lowercase")]
pub struct Config {
    programs: Option<Vec<Program>>,
    taskmasterd: Option<Taskmasterd>,
    taskmasterctl: Option<Taskmasterctl>,
    unix_http_server: Option<UnixHttpServer>,
    inet_http_server: Option<InetHttpServer>,
    include: Option<Include>,
    group: Option<Vec<Group>>,
    fcgi_program: Option<Vec<FcgiProgram>>,
    eventlistener: Option<Vec<EventListener>>,
    rpcinterface: Option<Vec<RpcInterface>>
}

/// Program structure is a section of Config in order to run a task.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Program {
    command: String,
    process_name: String,
    numprocs: u16,
    directory: PathBuf,
    umask: String, // https://docs.rs/umask/1.0.0/umask/
    priority: i32,
    autostart: bool
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
    stdout_capture_maxbytes: i32,
    stdout_events_enabled: bool,
    stderr_logfile: PathBuf,
    stderr_logfile_maxbytes: i32,
    stderr_logfile_backups: i32,
    stderr_capture_maxbytes: i32,
    stderr_events_enabled: bool,
    environment: hashMap<String, String>,
    serverurl: String
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
    Blather
}

/// The logging level, dictating what is written to the supervisord activity log.
/// One of critical, error, warn, info, debug, trace, or blather.
/// Note that at log level debug, the supervisord log file will record the
/// stderr/stdout output of its child processes and extended info info about
/// process state changes, which is useful for debugging a process which isn’t starting properly.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct Taskmasterd {
    logfile: PathBuf,
    logfile_maxbytes: i32,
    logfile_backups: i32,
    loglevel: LogLevel,
    pidfile: i32,
    nodaemon: bool,
    minfds: i32,
    minprocs: i32,
    umask: String,
    user: String
    identifier: String,
    directory: PathBuf,
    nocleanup: bool,
    childlogdir: PathBuf,
    strip_ansi: bool,
    environment: HashMap<String, String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct Taskmasterctl {
    serverurl: String,
    username: String,
    password: String,
    prompt: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct UnixHttpServer {
    file: String
    chmod: String
    chown: String,
    username: String,
    password: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct InetHttpServer {
    port: String, // IPV4 or IPV6 + PORT
    username: String,
    password: String
}

/// Files replace the order and values of LOOKAT
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct Include {
    files: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct Group {
    programs: String,
    priority: i32
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct FcgiProgram {
    command: String,
    socket=unix:///var/run/supervisor/%(program_name)s.sock
    socket_owner=chrism
    socket_mode=0700
    process_name: String,
    numprocs: u16,
    directory: PathBuf,
    umask: String, // https://docs.rs/umask/1.0.0/umask/
    priority: i32,
    autostart: bool
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
    environment: hashMap<String, String>,
    serverurl: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct EventListener {
    command: String,
    process_name: String,
    numprocs: u16,
    events=PROCESS_STATE
    buffer_size=10
    directory: PathBuf,
    umask: String, // https://docs.rs/umask/1.0.0/umask/
    priority: i32,
    autostart: bool
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
    environment: hashMap<String, String>,
    serverurl: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct RpcInterface {
    tasmkaster.rpcinterface_factory: String, // taskmaster.rpcinterface:make_main_rpcinterface
    retries: i32
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
        let file = File::open(&path)?;
        let deserialized_conf: Config = serde_yaml::from_reader(file)?;
        Ok(deserialized_conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_default_values() {}
}
