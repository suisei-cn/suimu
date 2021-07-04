pub mod types;

use crate::Music;
use csv::{Error, Reader};
use std::io::Read;

pub fn check_csv(source: impl Read) -> Result<Vec<Music>, String> {
    let mut reader = Reader::from_reader(source);
    match reader.deserialize().collect::<Result<Vec<Music>, Error>>() {
        Ok(data) => Ok(data),
        Err(err) => Err(err.to_string()),
    }
}
