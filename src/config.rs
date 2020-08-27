use std::path::{Path, PathBuf};

#[derive(Debug)]
/// Configuration structure used to run a task.
pub struct Config {
    /// Command to run.
    cmd: String,
    /// Number of processors this task should run with.
    // u16 is fine because we are not expecting a machine to run more
    // than 2^16 proc at any single time.
    numprocs: u16,
    /// Working directory of the task.
    workingdir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cmd: "ls".into(),
            numprocs: 1,
            workingdir: PathBuf::from("/tmp"),
        }
    }
}

impl std::convert::TryFrom<&Path> for Config {
    type Error = std::io::Error;

    fn try_from(_file_name: &Path) -> Result<Self, Self::Error> {
        // TODO: deserialize from yaml file
        Ok(Self::default())
    }
}
