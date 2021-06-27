pub mod utils;
use std::{io::Result, path::PathBuf, io::Error, io::ErrorKind};
use clap::ArgMatches;
use self::utils::{object::{self, ObjectFormat}, repo};

pub fn init(mut dir: PathBuf, args: &ArgMatches) -> Result<()> {
    if let Some(path) = args.value_of("PATH") {
        dir.push(path);
    }

    match repo::create(dir) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

pub fn cat_file(dir: PathBuf, args: &ArgMatches) -> Result<()> {
    let obj_sha = match args.value_of("OBJECT") {
        Some(sha) => sha,
        None => return Err(Error::new(ErrorKind::InvalidInput, "Must provide a object hash"))
    };

    let repo = repo::find(dir)?;
    let obj = object::read(repo, obj_sha)?;
    if let Some(data) = object::serialize(&obj) {
        println!("{}", data);
    }

    Ok(())
}

pub fn hash_file(mut dir: PathBuf, args: &ArgMatches) -> Result<()> {
    let obj_name = match args.value_of("OBJECT") {
        Some(n) => n,
        None => return Err(Error::new(ErrorKind::InvalidInput, "Must provide a object to hash"))
    };


    let obj_type = match args.value_of("type") {
        Some(t) => ObjectFormat::from_str(t)?,
        None => ObjectFormat::Blob
    };

    let write = args.is_present("write");
    let repository = match repo::find(dir.clone()) {
        Ok(r) => {
            if write {
                Some(r)
            } else {
                None
            }
        },
        Err(_) => {
            if write {
                return Err(Error::new(ErrorKind::NotFound, "Must be in valid kit repo to write hashed objects"));
            } else {
                None
            }
        }
    };

    dir.push(obj_name);
    let sha = object::hash(dir, obj_type, repository)?;
    println!("{}", sha);
    Ok(())
}
