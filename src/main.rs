mod config;

use config::Config;
use std::convert::TryFrom;
use std::path::Path;

fn main() -> Result<(), std::io::Error> {
    let path = Path::new("config.yaml");
    let config = Config::try_from(path)?;
    println!("Here's your config: {:#?}", config);
    Ok(())
}
