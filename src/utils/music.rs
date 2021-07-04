use crate::{MaybeMusic, PLATFORM_SUPPORTED};
use anyhow::{anyhow, Error, Result};
use chrono::{DateTime, FixedOffset};
use std::convert::TryFrom;
use std::hash::Hasher;
use twox_hash::XxHash64;

#[derive(Debug)]
pub struct Music {
    pub datetime: DateTime<FixedOffset>,
    pub video_type: String,
    pub video_id: String,
    pub clip_start: Option<f32>,
    pub clip_end: Option<f32>,
    pub status: u16,
    pub title: String,
    pub artist: String,
    pub performer: String,
    pub comment: String,
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>, Error> {
    if let Ok(v) = chrono::DateTime::parse_from_rfc3339(time) {
        return Ok(v);
    }
    // for back-compat reasons...
    if let Ok(v) = chrono::DateTime::parse_from_str(time, "%Y-%m-%dT%H:%M%z") {
        return Ok(v);
    }
    Err(anyhow!("Invalid datetime"))
}

fn to_string(x: Option<f32>) -> String {
    match x {
        None => "".to_string(),
        Some(v) => v.to_string(),
    }
}

impl Music {
    pub fn hash(&self) -> String {
        // https://github.com/suisei-cn/suisei-music/blob/6b5767b58eee61cf2bcdf2be60ac9e06c773809d/tools/mod.py#L28
        let mut hasher = XxHash64::with_seed(0x9f88f860);

        hasher.write(self.video_type.as_bytes());
        hasher.write(self.video_id.as_bytes());
        hasher.write(to_string(self.clip_start).as_bytes());
        hasher.write(to_string(self.clip_end).as_bytes());
        hasher.write(self.title.as_bytes());
        hasher.write(self.artist.as_bytes());
        hasher.write(self.performer.as_bytes());

        format!("{:016x}", hasher.finish())
    }
}

impl TryFrom<MaybeMusic> for Music {
    type Error = anyhow::Error;

    fn try_from(v: MaybeMusic) -> Result<Music> {
        let datetime = parse_time(&v.datetime)?;
        let status = v.status.ok_or_else(|| anyhow!("No status present"))?;
        let video_type = v.video_type.trim();
        let title = v.title.trim();

        if !PLATFORM_SUPPORTED.contains(&video_type) {
            return Err(anyhow!("Platform not supported"));
        }

        if title.is_empty() {
            return Err(anyhow!("Title is empty"));
        }

        Ok(Music {
            datetime,
            status,
            video_type: video_type.to_string(),
            video_id: v.video_id.trim().to_string(),
            clip_start: v.clip_start,
            clip_end: v.clip_end,
            title: title.to_string(),
            artist: v.artist.trim().to_string(),
            performer: v.performer.trim().to_string(),
            comment: v.comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_compat() {
        assert_eq!(
            (Music {
                datetime: DateTime::parse_from_rfc3339("2021-06-25T22:30:00+09:00").unwrap(),
                video_type: "YOUTUBE".to_string(),
                video_id: "ZfDYRy17CBY".to_string(),
                clip_start: None,
                clip_end: None,
                status: 0,
                title: "Bluerose".to_string(),
                artist: "星街すいせい".to_string(),
                performer: "星街すいせい".to_string(),
                comment: "".to_string(),
            })
            .hash(),
            "0c2b9da9cfe08c9e"
        );

        assert_eq!(
            (Music {
                datetime: DateTime::parse_from_rfc3339("2020-03-22T20:00:00+09:00").unwrap(),
                video_type: "YOUTUBE".to_string(),
                video_id: "vQHVGXdcqEQ".to_string(),
                clip_start: None,
                clip_end: None,
                status: 0,
                title: "NEXT COLOR PLANET".to_string(),
                artist: "星街すいせい".to_string(),
                performer: "星街すいせい".to_string(),
                comment: "".to_string(),
            })
            .hash(),
            "4db7f3845af9cce9"
        );
    }
}
