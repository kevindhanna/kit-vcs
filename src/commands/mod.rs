pub mod utils;
use std::{io::Result, path::PathBuf};
use clap::ArgMatches;
use self::utils::{join_path, repo};

pub fn init(current_dir: &str, args: &ArgMatches) -> Result<()> {
    let dir: String;
    if let Some(path) = args.value_of("PATH") {
        dir = join_path(current_dir, path)
    } else {
        dir = current_dir.to_owned();
    }

    repo::create(&dir)
}
