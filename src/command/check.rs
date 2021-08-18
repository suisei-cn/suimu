use crate::utils::{check_csv, check_logic};
use crate::{MaybeMusic, Platform};
use anyhow::{anyhow, ensure, Result};
use levenshtein::levenshtein;
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

/// Return the Levenshtein ratio of two strings. SHall be a value between 0 and 1.
fn similarity_ratio(a: &str, b: &str, len: usize) -> f32 {
    1f32 - (levenshtein(a, b) as f32) / (len as f32)
}

fn similarity_check(
    field_name: &str,
    musics: &[MaybeMusic],
    picker: impl for<'a> Fn(&'a MaybeMusic) -> &'a str,
) {
    for (i, one) in musics.iter().map(&picker).enumerate() {
        for two in musics.iter().map(&picker).skip(i + 1) {
            if one == two {
                continue;
            }
            let sim = similarity_ratio(one, two, one.chars().count().max(two.chars().count()));
            if sim > 0.75 {
                warn!(
                    "[{}] {} & {}: Similar titles ({})",
                    field_name, one, two, sim
                );
            }
        }
    }
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

    // Potential typo analysis
    info!("Checking potential typos...");
    for x in &check_result {
        if x.title.trim() != x.title {
            warn!("{}: Spaces around title", x);
            continue;
        }
        if x.artist.trim() != x.artist {
            warn!("{}: Spaces around artist", x);
            continue;
        }
    }

    info!("Check similar metadatas...");
    similarity_check("Title", &check_result, |x| &x.title);
    similarity_check("Artist", &check_result, |x| &x.artist);

    info!("Check finished.");

    if opts.json_output {
        let base = serde_json::to_string(&check_result).unwrap();
        println!("{}", base);
    }

    Ok(())
}
