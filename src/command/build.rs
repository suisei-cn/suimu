use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{ensure, Result};
use chrono::{DateTime, FixedOffset, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use structopt::{clap, StructOpt};

use crate::utils::{check_csv, process_music, rfc3339, EnvConf};
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

    #[structopt(long, about = "Target diff file")]
    output_diff: Option<PathBuf>,
}

#[derive(Serialize)]
struct OutputDiff<'a> {
    added: Vec<&'a OutputMusic>,
    removed: Vec<&'a OutputMusic>,
    #[serde(serialize_with = "rfc3339::serialize_utc")]
    last_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct OutputMusic {
    url: String,
    #[serde(with = "rfc3339::with_rfc3339")]
    datetime: DateTime<FixedOffset>,
    title: String,
    artist: String,
    performer: String,
    status: u16,
    source: String,
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
        output_diff: Option<PathBuf>,
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
            output_diff,
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

    let mut old_output = None;
    if opts.output_json.is_some() && opts.output_diff.is_some() {
        let output_json = opts.output_json.clone().unwrap();
        if !output_json.exists() {
            info!("Old output_json not found, assuming empty.");
        } else {
            let old_json = File::open(output_json)?;
            let was_output =
                serde_json::from_reader::<_, Vec<OutputMusic>>(BufReader::new(old_json));
            if let Err(x) = was_output {
                warn!("Failed to read old JSON, assuming empty: {:?}", x);
                old_output = Some(vec![]);
            } else {
                let list = was_output
                    .unwrap()
                    .into_iter()
                    .filter(|x| {
                        let mut path = output_dir.clone();
                        let filename = x.url.split('/').last().unwrap();
                        path.push(filename);
                        if path.exists() {
                            true
                        } else {
                            warn!(
                                "{} is present in the list, but the file is missing.",
                                filename
                            );
                            false
                        }
                    })
                    .collect();
                old_output = Some(list);
            }
        }
    }

    info!("=============== Starting build ===============");
    for (idx, i) in music_process_arr.iter().enumerate() {
        info!("======== Building {} / {} ========", idx + 1, length);
        process_music(i, &env_conf, &mut global_stat);
    }
    info!("=============== Finishing build ===============");

    if opts.output_json.is_some() {
        // Generate new output.
        let baseurl = opts.baseurl.unwrap();
        let new_output = music_arr
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
        let output_json_text = serde_json::to_string(&new_output)?;
        std::fs::write(opts.output_json.unwrap(), output_json_text)?;

        // Writing diff.
        if let Some(oop) = old_output {
            let removed = oop.iter().filter(|x| !new_output.contains(x)).collect();
            let added = new_output.iter().filter(|x| !oop.contains(x)).collect();
            let output_json_text = serde_json::to_string(&OutputDiff {
                removed,
                added,
                last_updated: Utc::now(),
            })?;
            std::fs::write(opts.output_diff.unwrap(), output_json_text)?;
        }
    }

    Ok(())
}
