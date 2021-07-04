use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::clap;
use structopt::StructOpt;
use suimu::utils::{check_csv, check_logic};
use suimu::Platform;

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
    #[structopt(
        about = "The CSV file to check",
        default_value = "suisei-music.csv",
        index = 1,
        required = true
    )]
    csv_file: PathBuf,

    #[structopt(short, long, about = "Only check formats")]
    format_only: bool,
}

fn main() {
    // Set default logging level to INFO
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Parse arguments
    let opts = Opt::from_args();
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

    if opts.format_only {
        return;
    }

    // Logic analysis
    info!("Checking entry logic...");
    for x in &arr {
        if let Err(v) = check_logic(x) {
            warn!("{}: {}", x, v);
        }
    }

    // Support analysis
    info!("Checking entry support...");
    for x in &arr {
        if x.video_type.is_empty() {
            warn!("{}: Empty video_type", x);
            continue;
        }
        if let Err(v) = Platform::from_str(&x.video_type) {
            warn!("{}: {}", x, v);
        }
    }

    info!("Check finished.");
}
