use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Music {
    datetime: String,
    video_type: String,
    video_id: String,
    clip_start: Option<f32>,
    clip_end: Option<f32>,
    status: Option<u16>,
    title: String,
    artist: String,
    performer: String,
    comment: String,
}
