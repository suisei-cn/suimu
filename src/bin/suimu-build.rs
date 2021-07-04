use std::convert::TryInto;
use std::fs::File;
use std::path::PathBuf;
use structopt::clap;
use structopt::StructOpt;
use suimu::utils::{check_csv, process_music};
use suimu::Music;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

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
}

fn main() {
    // Set default logging level to INFO
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Parse arguments
    let opts = Opt::from_args();

    // --csv-file, readable & parsable
    let csv_file: PathBuf = opts.csv_file;
    info!("CSV file: {:?}", csv_file);

    if !csv_file.exists() {
        error!("Cannot open CSV file.");
        std::process::exit(2);
    }

    let read_file = File::open(csv_file).unwrap();
    let check_result = check_csv(&read_file);

    if let Err(e) = check_result {
        error!("CSV validation failed: {}", e);
        std::process::exit(1);
    }

    let arr = check_result.unwrap();
    info!("CSV successfully validated. {} entries found.", arr.len());

    let mut music_arr = vec![];

    for x in arr {
        let music: Result<Music, _> = x.clone().try_into();
        match music {
            Ok(m) => {
                music_arr.push(m);
            }
            Err(e) => {
                if !x.video_id.is_empty() {
                    warn!("Skipping music {}: {}", x, e.to_string());
                }
            }
        }
    }

    let output_dir = opts.output_dir;

    info!("{} valid entries found.", music_arr.len());

    let music_process_arr: Vec<Music> = music_arr
        .into_iter()
        .filter(|x| {
            let mut dir: PathBuf = output_dir.clone();
            dir.push(format!("{}.m4a", x.hash()));
            !dir.exists()
        })
        .collect();

    info!("{} entries to process.", music_process_arr.len());

    for i in music_process_arr {
        process_music(i);
    }
}
