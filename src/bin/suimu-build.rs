use anyhow::{ensure, Result};
use std::convert::TryInto;
use std::fs::File;
use std::path::PathBuf;
use structopt::clap;
use structopt::StructOpt;
use suimu::utils::{check_csv, process_music, EnvConf};
use suimu::Music;

use log::{debug, info, warn};

#[derive(StructOpt)]
#[structopt(
version = clap::crate_version ! (),
author = clap::crate_authors ! (),
about = clap::crate_description ! ()
)]
struct Opt {
    #[structopt(short, long, about = "CSV file path", required = true)]
    csv_file: PathBuf,

    #[structopt(short, long, about = "Output directory", required = true)]
    output_dir: PathBuf,

    #[structopt(short, long, about = "Source directory", required = true)]
    source_dir: PathBuf,

    #[structopt(short, long, about = "Don't process musics")]
    dry_run: bool,

    #[structopt(long, about = "ffmpeg executable", default_value = "ffmpeg")]
    ffmpeg: String,

    #[structopt(long, about = "youtube-dl executable", default_value = "youtube-dl")]
    ytdl: String,
}

fn main() -> Result<()> {
    // Set default logging level to INFO
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Parse arguments
    let opts = Opt::from_args();

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

    let music_process_arr: Vec<Music> = music_arr
        .into_iter()
        .filter(|x| {
            let mut dir = output_dir.to_owned();
            dir.push(format!("{}.m4a", x.hash()));
            !dir.exists()
        })
        .collect();

    info!("{} entries to process.", music_process_arr.len());

    if opts.dry_run {
        info!("Dry run: music processing is skipped.");
    }

    let env_conf = EnvConf {
        source_dir: opts.source_dir,
        output_dir,
        youtube_dl_path: opts.ytdl,
        ffmpeg_path: opts.ffmpeg,
    };

    for i in music_process_arr {
        process_music(i, &env_conf);
    }

    Ok(())
}
