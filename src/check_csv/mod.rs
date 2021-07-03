pub mod types;

use crate::Music;
use csv::Reader;

pub fn check_csv(text: &str) -> Result<Vec<Music>, String> {
    let mut reader = Reader::from_reader(text.as_bytes());
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
