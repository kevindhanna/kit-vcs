use std::fs::File;
use std::path::PathBuf;
use std::default::Default;
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use flate2::Compression;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha::sha1::Sha1;
use sha::utils::{Digest, DigestExt};
use super::repo::{self, KitRepository};

trait IsObject {
    fn repo(&self) -> &KitRepository;
}

pub enum KitObject {
    Commit {
        repo: KitRepository,
        data: Option<String>,
    },
    Tree {
        repo: KitRepository,
        data: Option<String>
    },
    Tag {
        repo: KitRepository,
        data: Option<String>
    },
    Blob {
        repo: KitRepository,
        data: Option<String>
    }
}

impl IsObject for KitObject {
    fn repo(&self) -> &KitRepository {
        match self {
            KitObject::Commit { repo, data } => repo,
            KitObject::Tree{ repo, data } => repo,
            KitObject::Tag{ repo, data } => repo,
            KitObject::Blob{ repo, data } => repo,
        }
    }
}

pub fn serialize(obj: &KitObject) -> &Option<String>  {
    match obj {
        KitObject::Commit { repo, data } => data,
        KitObject::Tree{ repo, data } => data,
        KitObject::Tag{ repo, data } => data,
        KitObject::Blob{ repo, data } => data,
    }
}

pub fn deserialize(obj: KitObject, new_data: Option<String>) -> KitObject {
    match obj {
        KitObject::Commit { repo, data } => KitObject::Commit { repo, data: new_data },
        KitObject::Tree{ repo, data } => KitObject::Tree { repo, data: new_data },
        KitObject::Tag{ repo, data } => KitObject::Tag { repo, data: new_data },
        KitObject::Blob{ repo, data } => KitObject::Blob { repo, data: new_data },
    }
}

pub enum ObjectFormat {
    Commit,
    Tree,
    Tag,
    Blob
}
impl ObjectFormat {
    pub fn from_str(s: &str) -> Result<ObjectFormat> {
        match s {
            "commit" => Ok(ObjectFormat::Commit),
            "tree" => Ok(ObjectFormat::Tree),
            "tag" => Ok(ObjectFormat::Tag),
            "blob" => Ok(ObjectFormat::Blob),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid Object Format"))
        }
    }
    pub fn from_obj(obj: &KitObject) -> ObjectFormat {
        match obj {
            KitObject::Commit { repo, data } => ObjectFormat::Commit,
            KitObject::Tree{ repo, data } => ObjectFormat::Tree,
            KitObject::Tag{ repo, data } => ObjectFormat::Tag,
            KitObject::Blob{ repo, data } => ObjectFormat::Blob,
            _ => unreachable!()
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectFormat::Commit => "commit",
            ObjectFormat::Tree => "tree",
            ObjectFormat::Blob => "blob",
            ObjectFormat::Tag => "tag",
        }
    }
    pub fn as_string(&self) -> String {
        self.as_str().to_owned()
    }
}

pub fn write(obj: KitObject, actually_write: bool) -> Result<String> {
    let data: String;
    match serialize(&obj) {
        Some(d) => data = d.to_owned(),
        None => data = "".to_owned()
    }
    let format = ObjectFormat::from_obj(&obj);
    let mut result = format.as_string();
    result.push_str(&b' '.to_string());
    result.push_str(&data.len().to_string());
    result.push_str(&data);
    let sha = Sha1::default().digest(result.as_bytes()).to_hex();

    if actually_write {
        let path = repo::file(&obj.repo(), vec!("objects", &sha[0..2], &sha[2..]), actually_write)?;
        let mut file = File::create(path)?;
        let mut z = ZlibEncoder::new(&mut file, Compression::fast());
        z.write_all(result.as_bytes())?;
    }

    Ok(sha)
}

pub fn read(repo: repo::KitRepository , sha: &str) -> Result<KitObject> {
    let path = repo::file(&repo, vec!("objects", &sha[0..2], &sha[2..]), true).unwrap();

    let mut file = File::open(path.clone())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut z = ZlibDecoder::new(&buffer[..]);
    let mut data = String::new();
    z.read_to_string(&mut data)?;
    let x = data.find(' ').unwrap();
    let format = ObjectFormat::from_str(&data[0..x])?;

    let y = data.find('\x00').unwrap();
    let size = &data[(x + 1)..y];

    if size.parse::<usize>().unwrap() != data.len() - y - 1 {
        return Err(Error::new(ErrorKind::Other, format!("Malformed object {}: bad length", path.to_str().unwrap())));
    }

    let contents = String::from(&data[(y+1)..]);

    match format {
        ObjectFormat::Commit => {
            Ok(KitObject::Commit{repo, data: Some(contents)})
        },
        ObjectFormat::Tree => {
            Ok(KitObject::Tree{repo, data: Some(contents)})
        },
        ObjectFormat::Tag => {
            Ok(KitObject::Tag{repo, data: Some(contents)})
        },
        ObjectFormat::Blob => {
            Ok(KitObject::Blob{repo, data: Some(contents)})
        }
    }
}

pub fn hash(path: PathBuf, t: ObjectFormat, repo: Option<KitRepository>) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    match repo {
        Some(repo) => {
            let obj = match t {
                ObjectFormat::Blob => KitObject::Blob{ repo, data: Some(buf) },
                ObjectFormat::Commit => KitObject::Commit{ repo, data: Some(buf) },
                ObjectFormat::Tree => KitObject::Tree{ repo, data: Some(buf) },
                ObjectFormat::Tag => KitObject::Tag{ repo, data: Some(buf) },
            };
            println!("actually writing");
            return write(obj, true);
        }
        None => {
            let repo = repo::KitRepository::new(PathBuf::new(), true)?;
            let obj = match t {
                ObjectFormat::Blob => KitObject::Blob{ repo, data: Some(buf) },
                ObjectFormat::Commit => KitObject::Commit{ repo, data: Some(buf) },
                ObjectFormat::Tree => KitObject::Tree{ repo, data: Some(buf) },
                ObjectFormat::Tag => KitObject::Tag{ repo, data: Some(buf) },
            };
            println!("not writing");
            return write(obj, false)
        }
    }
}

pub fn find(repo: repo::KitRepository, name: String, format: ObjectFormat, follow: bool) -> String {
    return name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_stores_an_object() {
        let root = std::env::current_dir().unwrap();
        let mut dir = root.clone();
        dir.push("testing");
        dir.push("object");
        let repo = repo::create(dir.clone()).unwrap();
        let obj = KitObject::Blob{ repo, data: Some("Hello Joe".to_owned()) };
        let sha = self::write(obj, true).unwrap();
        let mut kit = dir.clone();
        kit.push(".kit");
        kit.push("objects");
        kit.push(&sha[0..2]);
        kit.push(&sha[2..]);
        assert!(kit.is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
