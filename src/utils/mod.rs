mod interactive;
mod maybemusic;
mod music;
mod process_music;

use std::io::Read;

use anyhow::{anyhow, ensure, Result};
use csv::{Error, Reader};
pub use interactive::*;
pub use maybemusic::MaybeMusic;
pub use music::Music;
pub use process_music::{process_music, EnvConf};
use serde::Serialize;
use strum_macros;

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
pub enum Platform {
    #[strum(serialize = "TWITTER")]
    Twitter,
    #[strum(serialize = "BILIBILI")]
    Bilibili,
    #[strum(serialize = "YOUTUBE")]
    YouTube,
}

pub fn check_csv(source: impl Read) -> Result<Vec<MaybeMusic>> {
    let mut reader = Reader::from_reader(source);
    reader
        .deserialize()
        .collect::<Result<Vec<MaybeMusic>, Error>>()
        .map_err(|err| anyhow!(err))
}

pub fn check_logic(x: &Music) -> Result<()> {
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
    use chrono::DateTime;

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

        assert!(
            check_csv(
                "video_type,video_id,clip_start,clip_end,status,title,artist,performer,comment
TWITTER,978601113791299585,,,0,Starduster,ジミーサムP,星街すいせい,"
                    .as_bytes()
            )
            .is_err()
        );
    }

    #[test]
    fn test_check_logic() {
        let common_dt = DateTime::parse_from_rfc3339("2021-06-25T22:30:00+09:00").unwrap();

        assert!(
            check_logic(&Music {
                datetime: common_dt,
                video_type: Platform::YouTube,
                video_id: "ZfDYRy17CBY".to_string(),
                clip_end: None,
                xxhash: "".to_string(),
                status: 0,
                title: "".to_string(),
                artist: "".to_string(),
                performer: "".to_string(),
                comment: "".to_string(),

                clip_start: Some(1.1),
            })
            .is_ok()
        );

        assert!(
            check_logic(&Music {
                datetime: common_dt,
                video_type: Platform::YouTube,
                video_id: "ZfDYRy17CBY".to_string(),
                xxhash: "".to_string(),
                status: 0,
                title: "".to_string(),
                artist: "".to_string(),
                performer: "".to_string(),
                comment: "".to_string(),

                clip_start: Some(3.1),
                clip_end: Some(2.2),
            })
            .is_err()
        );

        assert!(
            check_logic(&Music {
                datetime: common_dt,
                video_type: Platform::YouTube,
                video_id: "ZfDYRy17CBY".to_string(),
                xxhash: "".to_string(),
                status: 0,
                title: "".to_string(),
                artist: "".to_string(),
                performer: "".to_string(),
                comment: "".to_string(),

                clip_start: Some(1.1),
                clip_end: Some(2.2),
            })
            .is_ok()
        );
    }
}
