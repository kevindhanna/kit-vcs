use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use configparser::ini::Ini;

fn concat_str(str1: &str, str2: &str) -> String {
    let mut string = str1.to_owned();
    string.push_str(str2);
    return string;
}

pub fn join_path(str1: &str, str2: &str) -> String {
    concat_str(str1, &concat_str("/", str2))
}

pub mod repo {
    use super::*;
    pub fn path(repo: &Repository, name: &str) -> String {
        join_path(&repo.gitdir, name)
    }

    pub fn file(repo: &Repository, p: &str, mkdir: bool) -> Result<String, String> {
        let folders = p.split("/").collect::<Vec<&str>>();

        if let Ok(p) = dir(repo, &folders.join("/"), mkdir) {
            return Ok(path(repo, &p));
        }
        Err("Some err I guess".to_owned())
    }

    pub fn dir(repo: &Repository, p: &str, mkdir: bool) -> Result<String, String> {
        let p = path(repo, p);

        if dir_exists(&p) {
            return Ok(p);
        } else if file_exists(&p) {
            return Err(concat_str("Not a directory: ", &p));
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

    pub fn create(path: &str) -> io::Result<()> {
        if let Ok(repo) = Repository::new(path, true) {
            if !dir_exists(&repo.worktree) {
                let err = Error::new(ErrorKind::Other, concat_str(path, " is not a directory!"));
                return Err(err);
            }
            if !dir_empty(&repo.worktree) {
                let err = Error::new(ErrorKind::Other, concat_str(path, " is not a empty!"));
                return Err(err);
            }

            std::fs::create_dir_all(&repo.worktree).unwrap();
            for d in ["branches", "objects", "refs/tags", "refs/heads" ].to_vec() {
                dir(&repo, d, true).unwrap();
            }

            create_and_write_file(&repo.gitdir, "description", "Unnamed repository: edit this file 'description' to name the repository.\n")?;
            create_and_write_file(&repo.gitdir, "HEAD", "ref: refs/heads/master\n")?;
            create_and_write_file(&repo.gitdir, "config", &default_config())?;

            return Ok(())
        }
        let err = Error::new(ErrorKind::Other, "Failed to create repository struct".to_owned());
        Err(err)
    }

    // pub fn find(path: &str, required: bool) -> Repository {
    //     if let Ok(repo) = Repository::new(path, false) {
    //         return repo;
    //     }
    //     if let Ok(parent) = Path::new(path) {

    //         return find(parent, required);
    //     }
    // }

    pub struct Repository {
        pub worktree: String,
        pub gitdir: String,
        conf: HashMap<String, HashMap<String, Option<String>>>
    }

    impl Repository {
        pub fn new(path: &str, force: bool) -> Result<Repository, String> {
            let worktree = String::from(path);
            let gitdir = join_path(path, ".kit");
            let mut repo = Repository {
                worktree,
                gitdir,
                conf: HashMap::new()
            };

            if !(force || dir_exists(&repo.gitdir)) {
                return Err(concat_str("Path is not a Kit Repository: ", path));
            }

            let mut parser = Ini::new();
            let cf = repo::path(&repo, "config");
            if file_exists(&cf) {
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
    }
}

pub fn dir_exists(path: &String) -> bool {
    return Path::new(path).is_dir();
    // if let Ok(path_metadata) = fs::metadata(&dir) {
        // return path_metadata.is_dir();
    // }
    // return false;

}

pub fn dir_empty(path: &String) -> bool {
    PathBuf::from(path).read_dir().map(|mut i| i.next().is_none()).unwrap_or(false)
}

pub fn file_exists(path: &String) -> bool {
    return Path::new(path).is_file();
    // if let Ok(path_metadata) = fs::metadata(&dir) {
    //     return path_metadata.is_file();
    // }
    // return false;
}

fn create_and_write_file(path: &str, name: &str, contents: &str) -> io::Result<()> {
    let mut file = fs::File::create(join_path(&path, name))?;
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
