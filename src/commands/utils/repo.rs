use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use configparser::ini::Ini;

pub fn dir_empty(path: &PathBuf) -> bool {
    path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(false)
}

fn create_and_write_file(path: &PathBuf, name: &str, contents: &str) -> io::Result<()> {
    let mut path = path.clone();
    path.push(name);
    let mut file = fs::File::create(path)?;
    file.write(contents.as_bytes())?;
    Ok(())
}

pub fn file(repo: &KitRepository, mut folders: Vec<&str>, mkdir: bool) -> io::Result<PathBuf> {
    let name = folders.pop();
    if let Ok(mut p) = dir(repo, &folders.join("/"), mkdir) {
        if let Some(name) = name {
            p.push(name);
            return Ok(p);
        }
    }
    let err = Error::new(ErrorKind::Other, "Some err I guess");
    Err(err)
}

pub fn dir(repo: &KitRepository, name: &str, mkdir: bool) -> Result<PathBuf, String> {
    let mut p = repo.kitdir.clone();
    p.push(name);

    if p.exists() {
        if  p.is_dir() {
            return Ok(p);
        }
        return Err(format!("Not a directory: {}", p.to_str().unwrap()));
    }

    if mkdir {
        std::fs::create_dir_all(&p).unwrap();
        return Ok(p);
    }
    Err("Not making dirs".to_owned())
}

fn default_config() -> String {
    String::from("[core]
repositoryformatversion=0
filemode=false
bare=false\n")
}

pub fn create(path: PathBuf) -> io::Result<KitRepository> {
    if let Ok(repo) = KitRepository::new(path, true) {
        if !repo.worktree.exists() {
            std::fs::create_dir_all(&repo.worktree)?
        }
        if !repo.worktree.is_dir() {
            let err = Error::new(ErrorKind::Other, format!("{} is not a directory!", repo.workstree_string(), ));
            return Err(err);
        }
        if !dir_empty(&repo.worktree) {
            let err = Error::new(ErrorKind::Other, format!("{} is not a empty!", repo.workstree_string(), ));
            return Err(err);
        }

        std::fs::create_dir_all(&repo.worktree).unwrap();
        for d in ["branches", "objects", "refs/tags", "refs/heads" ].to_vec() {
            match dir(&repo, d, true) {
                Ok(_) => {},
                Err(s) => { return Err(Error::new(ErrorKind::Other, s)) }
            };
        }
        create_and_write_file(&repo.kitdir, "description", "Unnamed repository: edit this file 'description' to name the repository.\n")?;
        create_and_write_file(&repo.kitdir, "HEAD", "ref: refs/heads/master\n")?;
        create_and_write_file(&repo.kitdir, "config", &default_config())?;

        return Ok(repo)
    }
    let err = Error::new(ErrorKind::Other, "Failed to create repository struct".to_owned());
    Err(err)
}

pub fn find(path: PathBuf) -> io::Result<KitRepository> {
    if path.is_dir() {
        return KitRepository::new(path, false);
    }
    match path.parent() {
        Some(parent) => {
            return find(parent.to_path_buf());
        },
        None => {
            return Err(Error::new(ErrorKind::Other, "Not a kit directoy".to_owned()));
        }
    }
}

pub struct KitRepository {
    pub worktree: PathBuf,
    pub kitdir: PathBuf,
    conf: HashMap<String, HashMap<String, Option<String>>>
}

impl KitRepository {
    pub fn new(path: PathBuf, force: bool) -> io::Result<KitRepository> {
        let worktree = path.clone();
        let mut kitdir = path.clone();
        kitdir.push(".kit");
        let mut repo = KitRepository {
            worktree,
            kitdir,
            conf: HashMap::new()
        };

        if !(force || repo.kitdir.exists()) {
            let err = Error::new(ErrorKind::Other, format!("Path is not a Kit Repository: {}", path.to_str().unwrap()));
            return Err(err);
        }

        let mut parser = Ini::new();
        let mut cf = repo.kitdir.clone();
        cf.push("config");
        if cf.is_file() {
            let conf = parser.load(cf).unwrap();
            repo.conf = conf;
        } else if !force {
            let err = Error::new(ErrorKind::NotFound, "Configuration file missing".to_owned());
            return Err(err);
        }

        if !force {
            match parser.get("core", "repositoryformatversion") {
                Some(ver) => if ver != 0.to_string() {
                    let err = Error::new(ErrorKind::Other, format!("Unsupported repositoryformatversion: {}", &ver));
                    return Err(err);
                },
                None => {
                    let err = Error::new(ErrorKind::Other, "Config missing repositoryformatversion".to_owned());
                    return Err(err)
                },
            }
        }

        return Ok(repo);
    }

    pub fn workstree_string(&self) -> &str {
        self.worktree.to_str().unwrap()
    }

    pub fn kitdir_string(&self) -> &str {
        self.kitdir.to_str().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn assert_dir(mut path: PathBuf, dir: &str) {
        path.push(dir);
        assert!(path.is_dir());
    }

    fn assert_file(mut path: PathBuf, file: &str) {
        path.push(file);
        assert!(path.is_file());
    }

    #[test]
    fn it_creates_a_kit_repo() {
        let root = std::env::current_dir().unwrap();
        let mut dir = root.clone();
        dir.push("testing");
        dir.push("repo");
        create(dir.clone()).unwrap();
        let mut kit = dir.clone();
        kit.push(".kit");
        assert!(kit.is_dir());
        for d in ["branches", "objects", "refs", "refs/tags", "refs/heads"] {
            assert_dir(kit.clone(), d);
        };
        for f in ["description", "HEAD", "config"] {
            assert_file(kit.clone(), f);
        }
        fs::remove_dir_all(dir).unwrap();
    }
}
