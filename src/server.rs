use crate::{command::Command, config::Config, threadpool::ThreadPool, DEFAULT_ADDR};
use daemonize::Daemonize;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process,
    ffi::OsStr,
};
use users::{get_current_gid, get_current_uid};

/// Number of threads in the `ThreadPool`.
const NUM_THREADS: usize = 4;

/// Runs the server.
///
/// # Errors
///
/// Errors if it parsing the config file errors, or if binding to the default address fails.
pub fn run() -> Result<(), String> {
    let dir = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "Impossible to get user home directory".to_string())?;

    daemonize(&dir)?;

    let pool = ThreadPool::new(NUM_THREADS)?;

    let conf = Config::parse(None).unwrap_or_else(|err| {
        eprintln!(
            "{}: {}",
            env::args()
                .next()
                .as_ref()
                .map(Path::new)
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .map(String::from)
                .unwrap(),
            err
        );
        process::exit(1);
    });
    #[cfg(debug_assertions)]
    println! {"{:#?}", conf};


    let listener = TcpListener::bind(DEFAULT_ADDR).map_err(|e| format!("{:?}", e))?;

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => pool.execute(|| {
                let _ = handle_connection(tcp_stream);
            }),
            Err(e) => eprintln!("Error while listening for incoming messages: {:?}", e),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let mut buf = [0; 1024];

    match stream.read(&mut buf) {
        Ok(bytes) => {
            let _cmd: Command = serde_json::from_str(&String::from_utf8_lossy(&buf[..bytes]))
                .map_err(|e| format! {"Failed to deserialize Command: {:?}", e})?;
            // Execute the command here.

            // Answer back to client with command's status.
            stream
                .write_all(b"Your program is running ok.")
                .map_err(|e| format!("{:?}", e))?;
        }
        Err(e) => {
            eprintln!("Could not read from stream: {:?}", e);
        }
    }
    Ok(())
}

/// Daemonize the current program.
fn daemonize(home: &PathBuf) -> Result<(), String> {
    let stderr = File::create(home.join("taskmasterd.log")).map_err(|e| format!("{:?}", e))?;

    let daemonize = Daemonize::new()
        .pid_file(home.join("taskmasterd.pid"))
        .chown_pid_file(true)
        .working_directory(home)
        .user(get_current_uid())
        .group(get_current_gid())
        .umask(0o027)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
