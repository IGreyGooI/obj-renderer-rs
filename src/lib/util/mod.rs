use std::{
    convert,
    fs,
    io::{self, Read},
    num,
    path,
};

pub mod application_root;

pub fn load_file_as_u8<P: AsRef<path::Path>>(file_path: &P) -> Box<[u8]> {
    let mut buf = Vec::new();
    fs::File::open(file_path)
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    buf.into_boxed_slice()
}

pub trait Cyclic<T: Sized>
{
    fn index(&self) -> T;
    /// incrementing itself and then return the result
    fn increment(&mut self) -> T;
    /// decrementing itself and then return the result
    fn decrement(&mut self) -> T;
    /// incrementing itself and then return the result
    fn increment_by(&mut self, num: usize) -> T;
    /// decrementing itself and then return the result
    fn decrement_by(&mut self, num: usize) -> T;
}

#[derive(Clone, Debug, Default)]
pub struct CyclicIndex {
    pub index: usize,
    /// keep a clone of size to ensure locality at expense of double the memory use
    pub size: usize,
}

impl CyclicIndex {
    pub fn new(index: usize, size: usize) -> Self {
        CyclicIndex {
            index,
            size,
        }
    }
}

impl convert::From<CyclicIndex> for usize {
    fn from(cyclic_index: CyclicIndex) -> usize {
        cyclic_index.index.clone()
    }
}

impl Cyclic<usize> for CyclicIndex {
    fn index(&self) -> usize {
        self.index
    }
    fn increment(&mut self) -> usize {
        // It seems that this will hardly overflow but should be allow to overflow just in case
        self.index = (num::Wrapping(self.index) + num::Wrapping(1)).0 % self.size;
        self.index
    }
    fn decrement(&mut self) -> usize {
        // This will overflow and should be allow to overflow
        self.index = (num::Wrapping(self.index) - num::Wrapping(1)).0 % self.size;
        self.index
    }
    /// incrementing itself and then return the result
    fn increment_by(&mut self, num: usize) -> usize {
        self.index = (num::Wrapping(self.index) + num::Wrapping(num)).0 % self.size;
        self.index
    }
    /// decrementing itself and then return the result
    fn decrement_by(&mut self, num: usize) -> usize {
        self.index = (num::Wrapping(self.index) + num::Wrapping(num)).0 % self.size;
        self.index
    }
}

#[derive(Clone, Debug)]
pub struct History<T> {
    values: Box<[T]>,
    ptr: CyclicIndex,
    pub count: usize,
}

pub trait HistoryDefault {
    fn history_default() -> Self;
}

impl<T> History<T>
    where T: HistoryDefault + Clone {
    pub fn new(capacity: usize) -> Self {
        History {
            values: vec![T::history_default(); capacity].into_boxed_slice(),
            count: 0,
            ptr: CyclicIndex::new(0, capacity),
        }
    }
    
    pub fn clear(&mut self) {
        self.count = 0;
        self.ptr.index = 0;
    }
    
    pub fn push(&mut self, value: T) {
        let capacity = self.values.len();
        if self.count < capacity {
            self.count += 1;
        }
        self.values[self.ptr.index] = value;
        self.ptr.increment();
    }
}

pub struct HistoryIntoIterator<'a, T>
    where T: Clone + HistoryDefault {
    history: &'a History<T>,
    index: usize,
}

impl<'a, T> IntoIterator for &'a History<T>
    where T: Clone + HistoryDefault
{
    type Item = T;
    type IntoIter = HistoryIntoIterator<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        HistoryIntoIterator {
            history: self,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for HistoryIntoIterator<'a, T>
    where T: Clone + HistoryDefault
{
    type Item = T;
    
    fn next(&mut self) -> Option<T> {
        if self.index >= self.history.count {
            None
        } else {
            let len = self.history.values.len();
            let item = self.history.values[(len + self.history.ptr.index - 1 - self.index) % len]
                .clone();
            self.index += 1;
            Some(item)
        }
    }
}

pub struct Cycle<T: Copy> {
    items: Box<[T]>,
    index: usize,
}

impl<T> Cycle<T>
    where T: Copy
{
    pub fn new(items: &[T]) -> Cycle<T> {
        Cycle {
            items: items.to_vec().into_boxed_slice(),
            index: 0,
        }
    }
    
    pub fn get(&self) -> T { self.items[self.index] }
    
    pub fn next(&mut self) -> T {
        self.index = (self.index + 1) % self.items.len();
        self.items[self.index]
    }
    
    pub fn prev(&mut self) -> T {
        self.index = (self.index + self.items.len() - 1) % self.items.len();
        self.items[self.index]
    }
}
