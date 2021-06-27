pub mod utils;
use std::{io::Result, path::PathBuf};
use clap::ArgMatches;
use self::utils::repo;

pub fn init(mut dir: PathBuf, args: &ArgMatches) -> Result<()> {
    if let Some(path) = args.value_of("PATH") {
        dir.push(path);
    }

    repo::create(dir)
}
