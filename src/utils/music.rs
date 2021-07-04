use crate::{MaybeMusic, PLATFORM_SUPPORTED};
use anyhow::{anyhow, Error, Result};
use chrono::{DateTime, FixedOffset};
use std::convert::TryFrom;

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
