use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, FixedOffset};
use std::convert::TryFrom;
use std::str::FromStr;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::hash::Hasher;
use twox_hash::XxHash64;
use crate::maybemusic::MaybeMusic;
use crate::process_music::Platform;

#[derive(Debug)]
pub struct Music {
    pub datetime: DateTime<FixedOffset>,
    pub video_type: Platform,
    pub video_id: String,
    pub clip_start: Option<f32>,
    pub clip_end: Option<f32>,
    pub status: u16,
    pub title: String,
    pub artist: String,
    pub performer: String,
    pub comment: String,
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(time)
        .or_else(|_| DateTime::parse_from_str(time, "%Y-%m-%dT%H:%M%z"))
        .or_else(|_| bail!("Error parsing time"))
}

fn to_string(x: Option<f32>) -> String {
    x.map_or(String::new(), |x| x.to_string())
}

impl Music {
    pub fn hash(&self) -> String {
        // https://github.com/suisei-cn/suisei-music/blob/6b5767b58eee61cf2bcdf2be60ac9e06c773809d/tools/mod.py#L28
        let mut hasher = XxHash64::with_seed(0x9f88f860);

        hasher.write(self.video_type.as_ref().as_bytes());
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
        let video_type = Platform::from_str(v.video_type.trim())
            .map_err(|_| anyhow!("Platform not supported"))?;

        let title = v.title.trim();

        if title.is_empty() {
            return Err(anyhow!("Title is empty"));
        }

        Ok(Music {
            datetime,
            status,
            video_type,
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
    use crate::process_music::Platform;

    #[test]
    fn test_hash_compat() {
        assert_eq!(
            (Music {
                datetime: DateTime::parse_from_rfc3339("2021-06-25T22:30:00+09:00").unwrap(),
                video_type: Platform::YouTube,
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
                video_type: Platform::YouTube,
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

impl Display for Music {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let video_fmtid = format!("{}/{}", self.video_type.as_ref(), self.video_id);
        if self.title.is_empty() {
            return write!(f, "Untitled ({})", video_fmtid);
        }
        if self.artist.is_empty() {
            return write!(f, "{} ({})", self.title, video_fmtid);
        }
        write!(f, "{} - {} ({})", self.artist, self.title, video_fmtid)
    }
}