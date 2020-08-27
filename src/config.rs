use std::path::PathBuf;

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