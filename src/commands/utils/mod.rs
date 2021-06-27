use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use configparser::ini::Ini;

fn concat_str(str1: &str, str2: &str) -> String {
    let mut string = str1.to_owned();
    string.push_str(str2);
    return string;
}

pub mod repo {
    use super::*;
    pub fn file(repo: &Repository, p: &str, mkdir: bool) -> Result<PathBuf, String> {
        let mut folders = p.split("/").collect::<Vec<&str>>();
        let name = folders.pop();
        if let Ok(mut p) = dir(repo, &folders.join("/"), mkdir) {
            if let Some(name) = name {
                p.push(name);
                return Ok(p);
            }
        }
        Err("Some err I guess".to_owned())
    }

    pub fn dir(repo: &Repository, name: &str, mkdir: bool) -> Result<PathBuf, String> {
        let mut p = repo.kitdir.clone();
        p.push(name);

        if p.exists() {
          if  p.is_dir() {
              return Ok(p);
          }
            return Err(concat_str("Not a directory: ", p.to_str().unwrap()));
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

    pub fn create(path: PathBuf) -> io::Result<()> {
        if let Ok(repo) = Repository::new(path, true) {
            if !repo.worktree.exists() {
                std::fs::create_dir_all(&repo.worktree)?
            }
            if !repo.worktree.is_dir() {
                let err = Error::new(ErrorKind::Other, concat_str(repo.workstree_string(), " is not a directory!"));
                return Err(err);
            }
            if !dir_empty(&repo.worktree) {
                let err = Error::new(ErrorKind::Other, concat_str(repo.workstree_string(), " is not a empty!"));
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

            return Ok(())
        }
        let err = Error::new(ErrorKind::Other, "Failed to create repository struct".to_owned());
        Err(err)
    }

    pub fn find(path: PathBuf) -> Result<Repository, String> {
        if path.is_dir() {
            return Repository::new(path, false);
        }
        match path.parent() {
            Some(parent) => {
                return find(parent.to_path_buf());
            },
            None => {
                return Err("Not a kit directoy".to_owned());
            }
        }
    }

    pub struct Repository {
        pub worktree: PathBuf,
        pub kitdir: PathBuf,
        conf: HashMap<String, HashMap<String, Option<String>>>
    }

    impl Repository {
        pub fn new(path: PathBuf, force: bool) -> Result<Repository, String> {
            let worktree = path.clone();
            let mut kitdir = path.clone();
            kitdir.push(".kit");
            let mut repo = Repository {
                worktree,
                kitdir,
                conf: HashMap::new()
            };

            if !(force || repo.kitdir.exists()) {
                return Err(concat_str("Path is not a Kit Repository: ", "path"));
            }

            let mut parser = Ini::new();
            let mut cf = repo.worktree.clone();
            cf.push("config");
            if cf.is_file() {
                let conf = parser.load(cf)?;
                repo.conf = conf;
            } else if !force {
                return Err("Configuration file missing".to_owned());
            }

            if !force {
                match parser.get("core", "repositoryformatversion") {
                    Some(ver) => if ver != 0.to_string() {
                        return Err(concat_str("Unsupported repositoryformatversion: ", &ver));
                    },
                    None => return Err("Config missing repositoryformatversion".to_owned()),
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
}

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

#[cfg(test)]
mod tests {
    use super::*;

    mod repo_tests {
        use super::*;
        #[test]
        fn it_returns_a_full_repo_path() {
            let repo = repo::Repository::new("/mr/burns/and/me", true).unwrap();
            assert_eq!("/mr/burns/and/me/.kit/forever", repo::path(&repo, "forever"));
        }
    }
    #[test]
    fn it_returns_a_concatenated_string() {
        assert_eq!("Hello Joe", concat_str("Hello ", "Joe"));
    }

    #[test]
    fn it_returns_a_joined_path() {
        assert_eq!("My/Sharona", join_path("My", "Sharona"));
    }

}
