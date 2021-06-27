pub mod utils;
use std::{io::Result, path::PathBuf, io::Error, io::ErrorKind};
use clap::ArgMatches;
use self::utils::{object, repo};

pub fn init(mut dir: PathBuf, args: &ArgMatches) -> Result<()> {
    if let Some(path) = args.value_of("PATH") {
        dir.push(path);
    }

    repo::create(dir)
}

pub fn cat_file(dir: PathBuf, args: &ArgMatches) -> Result<()> {
    let repo = repo::find(dir)?;
    let obj_sha: &str;
    match args.value_of("OBJECT") {
        Some(sha) => obj_sha = sha,
        None => return Err(Error::new(ErrorKind::InvalidInput, "Must provide a object hash"))
    }

    let obj = object::read(repo, obj_sha)?;
    if let Some(data) = object::serialize(&obj) {
        println!("{}", data);
    }

    Ok(())
}
