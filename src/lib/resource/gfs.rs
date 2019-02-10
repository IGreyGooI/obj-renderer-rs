use std::{
    path,
    io,
    fmt,
    convert,
    collections::HashMap,
};
use super::ReadFile;
use super::super::util;

impl<T> fmt::Debug for GemFileSystem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLoader Path: {:#?}", self.root)
    }
}

impl<T> fmt::Display for GemFileSystem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLoader Path: {:#?}", self.root)
    }
}

pub struct Cache<T> {
    // storing the pointer of the file content: [T] in a HashMap
    pub indexer: HashMap<String, Box<[T]>>,
}

impl<T> Cache<T> {
    pub fn new() -> Cache<T> {
        Cache {
            indexer: HashMap::new(),
        }
    }
}

pub struct GemFileSystem<T> {
    pub cache: Cache<T>,
    pub root: path::PathBuf,
}

impl GemFileSystem<u8> {
    pub fn new<P: AsRef<path::Path>>(root: P) -> GemFileSystem<u8> {
        GemFileSystem {
            cache: Cache::new(),
            root: root.as_ref().to_path_buf(),
        }
    }
    /// load file into self.cache
    pub fn cache_file(&mut self, file_name: &String) {
        let mut path = self
            .root
            .clone();
        path.push(file_name.clone());
        
        match path.exists() && path.is_file() {
            true => {
                let file_ptr = util::load_file_as_u8(file_name);
                self.cache.indexer.insert(
                    file_name.clone(),
                    file_ptr.clone(),
                );
            }
            false => {}
        };
    }
    /// load and return file into self.cache
    pub fn fetch_and_cache_file(&mut self, file_name: &String) -> Option<Box<[u8]>> {
        let mut path = self.root.clone();
        let vec_string: Vec<&str> = file_name.split("/").collect();
        println!("{:#?}", vec_string);
        for string in vec_string{
            path.push(string);
        }
        println!("{:#?}", path);
        
        match path.exists() && path.is_file() {
            true => {
                let file_ptr = util::load_file_as_u8(&path);
                self.cache.indexer.insert(
                    file_name.clone(),
                    file_ptr.clone(),
                );
                Some(file_ptr)
            }
            false => {
                None
            }
        }
    }
}

impl ReadFile<u8> for GemFileSystem<u8> {
    fn load(&mut self, file_name: String) -> io::Result<Box<[u8]>> {
        let try_find_file = self.cache.indexer.get(&file_name);
        match try_find_file {
            None => {
                if let Some(file_ptr) = self.fetch_and_cache_file(&file_name) {
                    return Ok(file_ptr);
                } else {
                    // if reach here, it means it cannot find the file both in cache or in disk
                    let mut err = String::from("Resource not found at path: ");
                    err.push_str(&format!("{}", file_name));
                    Err(io::Error::new(io::ErrorKind::Other, err))
                }
            }
            Some(file) => {
                return Ok(file.clone());
            }
        }
    }
}

