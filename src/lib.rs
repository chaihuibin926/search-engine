use std::fs::OpenOptions;
use std::fs::File;
mod utils;
pub use utils::BloomFilter;

pub fn open_file(path: &str) -> File {
    return OpenOptions::new().read(true).write(true).create(true).append(true).open(path).unwrap();
}