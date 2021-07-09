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

pub fn check_logic(x: &MaybeMusic) -> Result<()> {
    // If clip start & end presents, make sure it's consistent
    if x.clip_start.is_some() && x.clip_end.is_some() {
        ensure!(
            (x.clip_start.unwrap() < x.clip_end.unwrap()),
            "clip_start is later than clip_end"
        )
    }

    Ok(())
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

    #[test]
    fn test_check_logic() {
        assert!(check_logic(&MaybeMusic::default()).is_ok());

        assert!(check_logic(&MaybeMusic {
            clip_start: Some(1.1),
            ..MaybeMusic::default()
        })
            .is_ok());

        assert!(check_logic(&MaybeMusic {
            clip_start: Some(3.1),
            clip_end: Some(2.2),
            ..MaybeMusic::default()
        })
            .is_err());

        assert!(check_logic(&MaybeMusic {
            clip_start: Some(1.1),
            clip_end: Some(2.2),
            ..MaybeMusic::default()
        })
            .is_ok());
    }
}
