use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::path::PathBuf;

use anyhow::{ensure, Result};
use chrono::{DateTime, FixedOffset};
use log::{debug, info, warn};
use serde::{Serialize, Serializer};
use structopt::{clap, StructOpt};

use crate::utils::{check_csv, process_music, EnvConf};
use crate::{Music, Platform};

const EXTENSION: &str = "m4a";

#[derive(StructOpt, Debug, Clone)]
#[structopt(
version = clap::crate_version ! (),
author = clap::crate_authors ! (),
about = "Build library from csv files"
)]
pub struct BuildOpt {
    #[structopt(short, long, about = "CSV file path", required = true)]
    csv_file: PathBuf,

    #[structopt(short, long, about = "Output directory", required = true)]
    output_dir: PathBuf,

    #[structopt(short, long, about = "Source directory", required = true)]
    source_dir: PathBuf,

    #[structopt(long, about = "Target JSON file", requires = "baseurl")]
    output_json: Option<PathBuf>,

    #[structopt(long, about = "Target JSON URL base")]
    baseurl: Option<String>,

    #[structopt(short, long, about = "Don't process musics")]
    dry_run: bool,

    #[structopt(long, about = "ffmpeg executable", default_value = "ffmpeg")]
    ffmpeg: String,

    #[structopt(long, about = "youtube-dl executable", default_value = "youtube-dl")]
    ytdl: String,
}

#[derive(Serialize)]
pub struct OutputMusic {
    url: String,
    #[serde(serialize_with = "serialize_rfc3339")]
    datetime: DateTime<FixedOffset>,
    title: String,
    artist: String,
    performer: String,
    status: u16,
    source: String,
}

pub fn serialize_rfc3339<S: Serializer>(
    val: &DateTime<FixedOffset>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&val.to_rfc3339())
}

impl OutputMusic {
    fn from(mu: &Music, baseurl: &str) -> Self {
        let info = &crate::utils::PLATFORM_INFO[&mu.video_type];
        let base_source = info.url_template.replace("{}", &mu.video_id);
        Self {
            url: baseurl
                .replacen("{}", &mu.xxhash, 1)
                .replacen("{}", EXTENSION, 1),
            datetime: mu.datetime,
            title: mu.title.clone(),
            artist: mu.artist.clone(),
            performer: mu.performer.clone(),
            status: mu.status,
            source: base_source,
        }
    }
}

pub struct GlobalStat {
    pub failed_video_items: HashSet<(Platform, String)>,
}

impl BuildOpt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        csv_file: PathBuf,
        output_dir: PathBuf,
        source_dir: PathBuf,
        dry_run: bool,
        ffmpeg: String,
        ytdl: String,
        output_json: Option<PathBuf>,
        baseurl: Option<String>,
    ) -> Self {
        Self {
            csv_file,
            output_dir,
            source_dir,
            dry_run,
            ffmpeg,
            ytdl,
            output_json,
            baseurl,
        }
    }
}

pub fn build(opts: BuildOpt) -> Result<()> {
    // --csv-file, readable & parsable
    let csv_file: PathBuf = opts.csv_file;
    debug!("CSV file: {:?}", csv_file);
    debug!("Output path: {:?}", opts.output_dir);
    debug!("Source path: {:?}", opts.source_dir);

    ensure!(csv_file.exists(), format!("{:?} does not exists", csv_file));

    let read_file = File::open(csv_file)?;
    let check_result = check_csv(&read_file)?;

    info!(
        "CSV successfully validated. {} entries found.",
        check_result.len()
    );

    let music_arr: Vec<Music> = check_result
        .into_iter()
        .filter_map(|x| {
            let empty_video_id = x.video_id.is_empty();
            let x_desc = &x.to_string();
            match TryInto::<Music>::try_into(x) {
                Ok(v) => Some(v),
                Err(e) => {
                    if !empty_video_id {
                        warn!("Skipping music {}: {}", x_desc, e.to_string());
                    }
                    None
                }
            }
        })
        .collect();

    let output_dir: PathBuf = opts.output_dir;

    info!("{} valid entries found.", music_arr.len());

    let music_process_arr = music_arr
        .iter()
        .filter(|x| {
            if x.is_member_only() {
                return false;
            }
            let mut dir = output_dir.to_owned();
            dir.push(format!("{}.m4a", x.xxhash));
            !dir.exists()
        })
        .collect::<Vec<_>>();

    info!("{} entries to process.", music_process_arr.len());

    if opts.dry_run {
        info!("Dry run: music processing is skipped.");
        return Ok(());
    }

    let env_conf = EnvConf {
        source_dir: opts.source_dir,
        output_dir: output_dir.clone(),
        youtube_dl_path: opts.ytdl,
        ffmpeg_path: opts.ffmpeg,
    };

    let mut global_stat = GlobalStat {
        failed_video_items: HashSet::new(),
    };

    let length = music_process_arr.len();

    info!("=============== Starting build ===============");
    for (idx, i) in music_process_arr.iter().enumerate() {
        info!("======== Building {} / {} ========", idx + 1, length);
        process_music(i, &env_conf, &mut global_stat);
    }
    info!("=============== Finishing build ===============");

    if opts.output_json.is_some() {
        let baseurl = opts.baseurl.unwrap();
        let output = music_arr
            .iter()
            .filter(|x| {
                if x.is_member_only() {
                    return false;
                }
                let filename = format!("{}.{}", x.xxhash, EXTENSION);
                let mut music_path = output_dir.clone();
                music_path.push(filename);
                if !music_path.exists() {
                    warn!("{} is not generated. Skipping.", x);
                    false
                } else {
                    true
                }
            })
            .map(|x| OutputMusic::from(x, &baseurl))
            .collect::<Vec<_>>();
        let output_json_text = serde_json::to_string(&output)?;
        std::fs::write(opts.output_json.unwrap(), output_json_text)?;
    }

    Ok(())
}
