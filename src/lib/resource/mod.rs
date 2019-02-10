use std::{
    io,
    path,
};

pub mod gfs;

pub trait ReadFile<T> {
    fn load(&mut self, file_name: String) -> io::Result<Box<[u8]>> ;
}
