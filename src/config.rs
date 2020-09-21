use std::{process, path::{Path, PathBuf}};
use signal::Signal;
use std::fs::File;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

/// Configuration structure used to run a task.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Config {
    /// Command to run.
    cmd: String,
    /// Number of processors this task should run with.
    // u16 is fine because we are not expecting a machine to run more
    // than 2^16 proc at any single time.
    numprocs: u16,
    /// Program mask running under
    umask: i32,
    /// Working directory of the task.
    workingdir: PathBuf,
    autostart: bool,
    autorestart: String,
    exitcodes: Vec<i32>,
    startretries: i32,
    starttime: i32,
    stopsignal: String,
    stoptime: i32,
    stdout: PathBuf,
    stderr: PathBuf,
    env: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let bin = "ls";
        let home = {
            match dirs::home_dir() {
                Some(dir) => dir,
                None => { eprintln!("Impossible to get user home directory!"); process::exit(1) }
            }
        };
        Self {
            cmd: bin.into(),
            numprocs: 1,
            umask: 0o027,
            workingdir: home.clone(),
            autostart: true,
            autorestart: "never".to_string(),
            exitcodes: { let mut v = Vec::new(); v.push(0); v },
            startretries: 0,
            starttime: 0,
         //   stopsignal: Signal::SIGTERM,
            stopsignal: "SIGTERM".to_string(),
            stoptime: 0,
            stdout: home.join(bin.to_string() + ".stdout"), // Would be lovely to replace "bin" by cmd from the struct Self
            stderr: home.join(bin.to_string() + ".stderr"), // Same
            env: HashMap::new(),
        }
    }
}

impl std::convert::TryFrom<&Path> for Config {
    type Error = std::io::Error;

    fn try_from(file_name: &Path) -> Result<Self, Self::Error> {
    // TODO: deserialize from yaml file
        let file = File::open(&file_name).expect("Unable to open");

        let d: Config = serde_yaml::from_reader(file).unwrap();

        println!("{:?}", d);

        Ok(Self::default())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_default_values() {
        let conf = Config::default();
        assert_eq!(conf.cmd, "ls");
        assert_eq!(conf.exitcodes[0], 0);
        assert_eq!(conf.workingdir, dirs::home_dir().unwrap());
        assert_eq!(conf.stdout, dirs::home_dir().unwrap().join("ls.stdout"));
        assert_eq!(conf.stderr, dirs::home_dir().unwrap().join("ls.stderr"));
    }

    #[test]
    fn prg_default_values() {
        let conf = Config { cmd: "nginx".to_string(), ..Default::default() };
        assert_eq!(conf.cmd, "nginx");
        assert_eq!(conf.exitcodes[0], 0);
        assert_eq!(conf.workingdir, dirs::home_dir().unwrap());
        assert_ne!(conf.stdout, dirs::home_dir().unwrap().join("ls.stdout"));
        assert_ne!(conf.stderr, dirs::home_dir().unwrap().join("ls.stderr"));
    }
}
