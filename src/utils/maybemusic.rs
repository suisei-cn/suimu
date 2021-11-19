use std::fmt::{Display, Formatter, Result as FmtResult};
use std::hash::Hasher;

use chrono::Local;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaybeMusic {
    pub datetime: String,
    pub video_type: String,
    pub video_id: String,
    pub clip_start: String,
    pub clip_end: String,
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
            clip_start: String::new(),
            clip_end: String::new(),
            status: None,
            title: String::new(),
            artist: String::new(),
            performer: String::new(),
            comment: String::new(),
        }
    }
}

impl MaybeMusic {
    pub fn hash(&self) -> String {
        // https://github.com/suisei-cn/suisei-music/blob/6b5767b58eee61cf2bcdf2be60ac9e06c773809d/tools/mod.py#L28
        let mut hasher = XxHash64::with_seed(0x9f88f860);

        hasher.write(self.video_type.as_bytes());
        hasher.write(self.video_id.as_bytes());
        hasher.write(self.clip_start.as_bytes());
        hasher.write(self.clip_end.as_bytes());
        hasher.write(self.title.as_bytes());
        hasher.write(self.artist.as_bytes());
        hasher.write(self.performer.as_bytes());

        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_compat() {
        assert_eq!(
            (MaybeMusic {
                datetime: "2021-06-25T22:30:00+09:00".to_string(),
                video_type: "YOUTUBE".to_string(),
                video_id: "ZfDYRy17CBY".to_string(),
                clip_start: "".to_string(),
                clip_end: "".to_string(),
                status: Some(0),
                title: "Bluerose".to_string(),
                artist: "星街すいせい".to_string(),
                performer: "星街すいせい".to_string(),
                comment: "".to_string(),
            })
            .hash(),
            "0c2b9da9cfe08c9e"
        );

        assert_eq!(
            (MaybeMusic {
                datetime: "2020-03-22T20:00:00+09:00".to_string(),
                video_type: "YOUTUBE".to_string(),
                video_id: "vQHVGXdcqEQ".to_string(),
                clip_start: "".to_string(),
                clip_end: "".to_string(),
                status: Some(0),
                title: "NEXT COLOR PLANET".to_string(),
                artist: "星街すいせい".to_string(),
                performer: "星街すいせい".to_string(),
                comment: "".to_string(),
            })
            .hash(),
            "4db7f3845af9cce9"
        );

        assert_eq!(
            (MaybeMusic {
                datetime: "2020-01-31T19:58+09:00".to_string(),
                video_type: "BILIBILI".to_string(),
                video_id: "BV1U7411s7X1".to_string(),
                clip_start: "971.0".to_string(),
                clip_end: "1194.8".to_string(),
                status: Some(0),
                title: "ホワイトハッピー".to_string(),
                artist: "極悪P".to_string(),
                performer: "星街すいせい".to_string(),
                comment: "".to_string(),
            })
            .hash(),
            "d52a8a351014118c"
        );
    }
}
