use clap::Clap;
use suimu::Music;
use suimu::{check_csv, PLATFORM_SUPPORTED};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[derive(Clap)]
#[clap(
version = clap::crate_version ! (),
author = clap::crate_authors ! (),
about = clap::crate_description ! ()
)]
struct Opts {
    #[clap(
        about = "The CSV file to check",
        default_value = "suisei-music.csv",
        index = 1,
        required = true
    )]
    csv_file: String,

    #[clap(short, long, about = "Only check formats")]
    format_only: bool,
}

fn check_support(x: &Music) -> Result<(), &str> {
    if !PLATFORM_SUPPORTED.contains(&&*x.video_type) {
        return Err("Platform not supported");
    }
    return Ok(());
}

fn check_logic(x: &Music) -> Result<(), &str> {
    // clip_start & clip_end existence
    if x.clip_start.is_none() ^ x.clip_end.is_none() == true {
        return Err("Only one of clip_start or clip_end exists");
    }

    if x.clip_start.is_some() & x.clip_end.is_some() {
        if x.clip_start.unwrap() > x.clip_end.unwrap() {
            return Err("clip_start is later than clip_end");
        }
    }
    return Ok(());
}

fn main() {
    // Set default logging level to INFO
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Parse arguments
    let opts: Opts = Opts::parse();
    let csv_file = opts.csv_file;
    info!("CSV file: {}", csv_file);

    let read_file = std::fs::read_to_string(csv_file);

    if read_file.is_err() {
        error!("Cannot open CSV file.");
        std::process::exit(2);
    }

    let read_file = read_file.unwrap();

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
        if let Err(v) = check_support(x) {
            warn!("{}: {}", x, v);
        }
    }
}