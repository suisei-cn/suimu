mod maybemusic;
mod music;

use csv::{Error, Reader};
use std::io::Read;

pub use maybemusic::MaybeMusic;
pub use music::Music;

pub const PLATFORM_SUPPORTED: [&str; 4] = ["TWITTER", "BILIBILI", "YOUTUBE", ""];

pub fn check_csv(source: impl Read) -> Result<Vec<MaybeMusic>, String> {
    let mut reader = Reader::from_reader(source);
    match reader
        .deserialize()
        .collect::<Result<Vec<MaybeMusic>, Error>>()
    {
        Ok(data) => Ok(data),
        Err(err) => Err(err.to_string()),
    }
}

pub fn check_logic(x: &MaybeMusic) -> Result<(), &str> {
    // clip_start & clip_end existence
    if x.clip_start.is_none() ^ x.clip_end.is_none() {
        return Err("Only one of clip_start or clip_end exists");
    }

    if x.clip_start.is_some()
        && x.clip_end.is_some()
        && (x.clip_start.unwrap() > x.clip_end.unwrap())
    {
        return Err("clip_start is later than clip_end");
    }
    Ok(())
}
