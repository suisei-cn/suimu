use chrono::Local;
use serde::{Serialize, Deserialize};
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaybeMusic {
    pub datetime: String,
    pub video_type: String,
    pub video_id: String,
    pub clip_start: Option<f32>,
    pub clip_end: Option<f32>,
    pub status: Option<u16>,
    pub title: String,
    pub artist: String,
    pub performer: String,
    pub comment: String,
}

impl Display for MaybeMusic {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let video_fmtid = if self.video_type.is_empty() {
            format!("paid, {}", self.datetime)
        } else {
            format!("{}/{}", self.video_type, self.video_id)
        };
        if self.title.is_empty() {
            return write!(f, "Untitled ({})", video_fmtid);
        }
        if self.artist.is_empty() {
            return write!(f, "{} ({})", self.title, video_fmtid);
        }
        write!(f, "{} - {} ({})", self.artist, self.title, video_fmtid)
    }
}

impl Default for MaybeMusic {
    fn default() -> Self {
        MaybeMusic {
            datetime: Local::now().to_rfc3339(),
            video_type: String::new(),
            video_id: String::new(),
            clip_start: None,
            clip_end: None,
            status: None,
            title: String::new(),
            artist: String::new(),
            performer: String::new(),
            comment: String::new(),
        }
    }
}