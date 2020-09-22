use daemonize::Daemonize;
use std::{
    convert::TryFrom,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process,
};
use taskmaster::{command::Command, config::Config, DEFAULT_ADDR};
use users::{get_current_gid, get_current_uid};

fn daemonize(home: &PathBuf) {
    let stderr = File::create(home.join("taskmasterd.log")).unwrap();

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
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error, {}", e);
            process::exit(1)
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let dir = dirs::home_dir();
    if dir.is_none() {
        eprintln!("Impossible to get user home directory!");
        process::exit(1);
    }
    daemonize(&dir.unwrap());
    let path = Path::new("config.yaml");
    let _config = Config::try_from(path)?;
    let listener = TcpListener::bind(DEFAULT_ADDR)?;

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("{:?}", handle_connection(tcp_stream));
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let mut buf = [0; 1024];

    if let Ok(bytes) = stream.read(&mut buf) {
        let _cmd: Command = serde_json::from_str(&String::from_utf8_lossy(&buf[..bytes]))
            .map_err(|e| e.to_string())?;
        // Execute the command here.

        // Answer back to client with command's status.
        stream
            .write_all(b"Your program is running ok.")
            .map_err(|e| e.to_string())?;
    } else {
        eprintln!("Could not read from stream.");
    }
    Ok(())
}
