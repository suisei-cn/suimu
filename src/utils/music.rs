use crate::{MaybeMusic, Platform};
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
#[derive(Debug, Serialize)]
pub struct Music {
    #[serde(serialize_with = "serialize_3339")]
    pub datetime: DateTime<FixedOffset>,
    pub video_type: Platform,
    pub video_id: String,
    pub clip_start: Option<f32>,
    pub clip_end: Option<f32>,
    pub xxhash: String,
    pub status: u16,
    pub title: String,
    pub artist: String,
    pub performer: String,
    pub comment: String,
}

pub fn serialize_3339<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.to_rfc3339();
    serializer.serialize_str(&s)
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(time)
        .or_else(|_| DateTime::parse_from_str(time, "%Y-%m-%dT%H:%M%z"))
        .or_else(|_| bail!("Error parsing time"))
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

        let parsed_clip_start = if v.clip_start.is_empty() {
            None
        } else {
            Some(v.clip_start.parse::<f32>()?)
        };

        let parsed_clip_end = if v.clip_start.is_empty() {
            None
        } else {
            Some(v.clip_end.parse::<f32>()?)
        };

        let xxhash = v.hash();

        Ok(Music {
            datetime,
            status,
            video_type,
            video_id: v.video_id.trim().to_string(),
            title: title.to_string(),
            artist: v.artist.trim().to_string(),
            performer: v.performer.trim().to_string(),
            comment: v.comment,
            xxhash,
            clip_start: parsed_clip_start,
            clip_end: parsed_clip_end,
        })
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
