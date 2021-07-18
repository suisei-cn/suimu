use crate::utils::{check_csv, check_logic};
use crate::Platform;
use anyhow::{anyhow, ensure, Result};
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::clap;
use structopt::StructOpt;

use log::{info, warn};

#[derive(StructOpt)]
#[structopt(
version = clap::crate_version ! (),
author = clap::crate_authors ! (),
about = "Validate csv files"
)]
pub struct CheckOpt {
    #[structopt(
        about = "The CSV file to check",
        default_value = "suisei-music.csv",
        index = 1,
        required = true
    )]
    csv_file: PathBuf,

    #[structopt(short, long, about = "Only check formats")]
    format_only: bool,

    #[structopt(long)]
    json_output: bool,
}

pub fn check(opts: CheckOpt) -> Result<()> {
    let csv_file: PathBuf = opts.csv_file;
    info!("CSV file: {:?}", csv_file);

    ensure!(csv_file.exists(), format!("{:?} does not exists", csv_file));

    let read_file = File::open(csv_file).unwrap();
    let check_result =
        check_csv(&read_file).map_err(|e| anyhow!(format!("CSV validation failed: {}", e)))?;

    info!(
        "CSV successfully validated. {} entries found.",
        check_result.len()
    );

    if opts.format_only {
        return Ok(());
    }

    // Logic analysis
    info!("Checking entry logic...");
    for x in &check_result {
        if let Err(v) = check_logic(x) {
            warn!("{}: {}", x, v);
        }
    }

    // Support analysis
    info!("Checking entry support...");
    for x in &check_result {
        if x.video_type.is_empty() {
            warn!("{}: Empty video_type", x);
            continue;
        }
        if let Err(v) = Platform::from_str(&x.video_type) {
            warn!("{}: {}", x, v);
        }
    }

    info!("Check finished.");

    if opts.json_output {
        let base = serde_json::to_string(&check_result).unwrap();
        println!("{}", base);
    }

    Ok(())
}
