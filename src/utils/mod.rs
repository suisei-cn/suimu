mod maybemusic;
mod music;
mod process_music;

use csv::{Error, Reader};
use std::io::Read;

pub use maybemusic::MaybeMusic;
pub use music::Music;
pub use process_music::process_music;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn return_sample_maybe_music() -> MaybeMusic {
        MaybeMusic {
            datetime: "2020-03-22T00:00:00+09:00".to_string(),
            video_type: "".to_string(),
            video_id: "".to_string(),
            clip_start: None,
            clip_end: None,
            status: None,
            title: "".to_string(),
            artist: "".to_string(),
            performer: "".to_string(),
            comment: "".to_string(),
        }
    }

    #[test]
    fn test_check_csv() {
        let ret1 = check_csv(
            "datetime,video_type,video_id,clip_start,clip_end,status,title,artist,performer,comment
2018-03-27T20:54+09:00,TWITTER,978601113791299585,,,0,Starduster,ジミーサムP,星街すいせい,"
                .as_bytes(),
        );

        assert_eq!(ret1.is_ok(), true);
        assert_eq!(ret1.unwrap().len(), 1);

        assert_eq!(
            check_csv(
                "video_type,video_id,clip_start,clip_end,status,title,artist,performer,comment
TWITTER,978601113791299585,,,0,Starduster,ジミーサムP,星街すいせい,"
                    .as_bytes()
            )
            .is_ok(),
            false
        );
    }

    #[test]
    fn test_check_logic() {
        let sample_mm = return_sample_maybe_music();

        assert_eq!(check_logic(&sample_mm).is_ok(), true);

        assert_eq!(
            check_logic(&MaybeMusic {
                clip_start: Some(1.1),
                ..sample_mm.clone()
            })
            .is_ok(),
            false
        );

        assert_eq!(
            check_logic(&MaybeMusic {
                clip_start: Some(3.1),
                clip_end: Some(2.2),
                ..sample_mm.clone()
            })
            .is_ok(),
            false
        );

        assert_eq!(
            check_logic(&MaybeMusic {
                clip_start: Some(1.1),
                clip_end: Some(2.2),
                ..sample_mm.clone()
            })
            .is_ok(),
            true
        );
    }
}
