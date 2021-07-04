pub mod types;

use crate::Music;
use csv::Reader;
use std::io::Read;

pub fn check_csv(source: impl Read) -> Result<Vec<Music>, String> {
    let mut reader = Reader::from_reader(source);
    let mut ret = vec![];

    for result in reader.deserialize() {
        match result {
            Ok(v) => {
                ret.push(v);
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }

    Ok(ret)
}
