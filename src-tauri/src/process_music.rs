use lazy_static::lazy_static;
use log::info;
use strum_macros::{AsRefStr, EnumString};

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use crate::music::Music;

pub struct PlatformSettings {
    url_template: &'static str,
    format: &'static str,
    source_ext: &'static str,
}

#[derive(Debug, Eq, Hash, PartialEq, AsRefStr, EnumString)]
pub enum Platform {
    #[strum(serialize = "TWITTER")]
    Twitter,
    #[strum(serialize = "BILIBILI")]
    Bilibili,
    #[strum(serialize = "YOUTUBE")]
    YouTube,
}

lazy_static! {
    pub static ref PLATFORM_INFO: HashMap<Platform, PlatformSettings> = {
        let mut m = HashMap::new();
        m.insert(
            Platform::YouTube,
            PlatformSettings {
                url_template: "https: //www.youtube.com/watch?v={}",
                format: "bestaudio[ext=m4a]",
                source_ext: "mp4",
            },
        );
        m.insert(
            Platform::Twitter,
            PlatformSettings {
                url_template: "https://www.twitter.com/i/status/{}",
                format: "best[ext=mp4]",
                source_ext: "mp4",
            },
        );
        m.insert(
            Platform::Bilibili,
            PlatformSettings {
                url_template: "https://www.bilibili.com/video/{}",
                format: "best[ext=flv]",
                source_ext: "flv",
            },
        );
        m
    };
}

pub struct EnvConf {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub youtube_dl_path: String,
    pub ffmpeg_path: String,
}

pub fn process_music(i: Music, conf: &EnvConf) {
    let info = &PLATFORM_INFO[&i.video_type];

    let mut source_path = conf.source_dir.clone();
    source_path.push(format!("{}.{}", i.video_id, info.source_ext));

    if !source_path.exists() {
        info!("Downloading {}", i);
        Command::new(&conf.youtube_dl_path)
            .arg("-f")
            .arg(info.format)
            .arg("-o")
            .arg(&source_path)
            .arg(info.url_template.replace("{}", &i.video_id))
            .output()
            .expect("Failed to execute youtube-dl");
    } else {
        info!("Skipping download: found {:?}", source_path);
    }

    let mut output_path = conf.output_dir.clone();
    output_path.push(format!("{}.{}", i.hash(), "m4a"));

    if !output_path.exists() {
        info!("Converting {}", i);
        let mut ffmpeg_cmd = Command::new(&conf.ffmpeg_path);
        ffmpeg_cmd
            .arg("-i")
            .arg(&source_path)
            .arg("-acodec")
            .arg("copy")
            .arg("-movflags")
            .arg("faststart")
            .arg("-metadata")
            .arg(format!("title={} / {}", i.title, i.artist))
            .arg("-metadata")
            .arg(format!("artist={}", i.performer))
            .arg("-vn");

        if i.clip_start.is_some() {
            ffmpeg_cmd.arg("-ss").arg(i.clip_start.unwrap().to_string());
        }
        if i.clip_end.is_some() {
            ffmpeg_cmd.arg("-to").arg(i.clip_end.unwrap().to_string());
        }

        ffmpeg_cmd
            .arg(output_path)
            .output()
            .expect("Failed to execute ffmpeg");
    } else {
        info!("Skipping conversion: found {:?}", output_path);
    }
}