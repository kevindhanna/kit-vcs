use std::fs::File;
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
    pub fn from_str(s: &str) -> ObjectFormat {
        match s {
            "commit" => ObjectFormat::Commit,
            "tree" => ObjectFormat::Tree,
            "tag" => ObjectFormat::Tag,
            "blob" => ObjectFormat::Blob,
            _ => unreachable!()
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
        let path = repo::file(&obj.repo(), vec!("objects", &sha[0..=2], &sha[2..]), actually_write)?;
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
    let format = ObjectFormat::from_str(&data[0..x]);

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
        _ => Err(Error::new(ErrorKind::Other, format!("Unkown type {} for object {}", format.as_string(), sha)))
    }
}

pub fn find(repo: repo::KitRepository, name: String, format: ObjectFormat, follow: bool) -> String {
    return name
}
