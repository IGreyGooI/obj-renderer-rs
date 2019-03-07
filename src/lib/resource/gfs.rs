use std::{
    collections::HashMap,
    convert,
    fmt,
    io,
    path,
};
use std::io::Cursor;
use std::io::Read;

use sha2::{Digest, Sha256};

use super::ReadFile;
use super::super::util;

const BUFFER_SIZE: usize = 1024;

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
    pub sha_2: HashMap<String, Vec<u8>>,
}

impl<T> Cache<T> {
    pub fn new() -> Cache<T> {
        Cache {
            indexer: HashMap::new(),
            sha_2: HashMap::new(),
        }
    }
}

pub struct GemFileSystem<T> {
    pub cache: Cache<T>,
    pub root: path::PathBuf,
}

pub enum FileSyncState {
    HashMatch,
    HashUnmatch,
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
                let mut file_ptr = util::load_file_as_u8(file_name);
                self.cache.indexer.insert(
                    file_name.clone(),
                    file_ptr.clone(),
                );
                self.cache.sha_2.insert(
                    file_name.clone(),
                    process::<Sha256, _>(&mut Cursor::new(file_ptr)),
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
        for string in vec_string {
            path.push(string);
        }
        println!("{:#?}", path);
    
        match path.exists() & &path.is_file() {
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
    
    pub fn check_for_sync_file(&mut self, file_name: String) -> io::Result<FileSyncState> {
        let result_if_file_in_cache = self.cache.indexer.get(&file_name);
        match result_if_file_in_cache {
            None => {
                let mut err = String::from("Resource not found in cache, cannot check for \
                synchronicity");
                err.push_str(&format!("{}", file_name));
                Err(io::Error::new(io::ErrorKind::Other, err))
            }
            Some(cached_file) => {
                let mut path = self
                    .root
                    .clone();
                path.push(file_name.clone());
                
                if path.exists() && path.is_file() {
                    let disk_file = util::load_file_as_u8(&file_name);
                    let disk_file_hash = process::<Sha256, _>(&mut Cursor::new(disk_file));
                    let cached_file_hash = self.cache.sha_2.get(&file_name).unwrap();
                    let diff_count = disk_file_hash
                        .iter()
                        .zip(cached_file_hash.iter())
                        .filter(|&
                                 (a, b)| a
                            != b).count();
                    if diff_count == 0 {
                        return Ok(FileSyncState::HashMatch);
                    } else {
                        return Ok(FileSyncState::HashUnmatch);
                    }
                } else {
                    let mut err = String::from("Resource not found at path: ");
                    err.push_str(&format!("{}", file_name));
                    Err(io::Error::new(io::ErrorKind::Other, err))
                }
            }
        }
    }
}

impl ReadFile<u8> for GemFileSystem<u8> {
    fn load(&mut self, file_name: String) -> io::Result<Box<[u8]>> {
        let result_if_file_in_cache = self.cache.indexer.get(&file_name);
        match result_if_file_in_cache {
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


fn process<D: Digest + Default, R: Read>(reader: &mut R) -> Vec<u8> {
    let mut sh = D::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => panic!(),
        };
        sh.input(&buffer[..n]);
        if n == 0 || n < BUFFER_SIZE {
            break;
        }
    }
    sh.result().to_vec()
}