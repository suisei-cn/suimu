use crate::maybemusic::MaybeMusic;

use anyhow::{anyhow, ensure, Result};
use csv::{Error, Reader};

use std::io::Read;

pub fn check_csv(source: impl Read) -> Result<Vec<MaybeMusic>> {
  let mut reader = Reader::from_reader(source);
  reader
    .deserialize()
    .collect::<Result<Vec<MaybeMusic>, Error>>()
    .map_err(|err| anyhow!(err))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_check_csv() {
    let ret1 = check_csv(
      "datetime,video_type,video_id,clip_start,clip_end,status,title,artist,performer,comment
2018-03-27T20:54+09:00,TWITTER,978601113791299585,,,0,Starduster,ジミーサムP,星街すいせい,"
        .as_bytes(),
    );

    assert!(ret1.is_ok());
    assert_eq!(ret1.unwrap().len(), 1);

    assert!(check_csv(
      "video_type,video_id,clip_start,clip_end,status,title,artist,performer,comment
TWITTER,978601113791299585,,,0,Starduster,ジミーサムP,星街すいせい,"
        .as_bytes()
    )
    .is_err());
  }
}
