use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, ensure, Result};
use lazy_static::lazy_static;
use levenshtein::levenshtein;
use log::{error, info, warn};
use regex::Regex;
use structopt::{clap, StructOpt};
use unicode_normalization::{is_nfc, UnicodeNormalization};

use crate::utils::{check_csv, check_logic};
use crate::{MaybeMusic, Music, Platform};

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

/// Return the Levenshtein ratio of two strings. SHall be a value between 0 and
/// 1.
fn similarity_ratio(a: &str, b: &str) -> f32 {
    let len = a.chars().count().max(b.chars().count());
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
            let sim = similarity_ratio(one, two);
            if sim > 0.75 {
                warn!(
                    "[{}] {} & {}: Similar titles ({})",
                    field_name, one, two, sim
                );
            }
        }
    }
}

lazy_static! {
    static ref RE: Regex = Regex::new(r" ?[（\(].+[）\)]$").unwrap();
}

pub fn check(opts: CheckOpt) -> Result<()> {
    let mut has_err = false;

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

    // Support analysis
    info!("Checking entry support...");
    for x in &check_result {
        if x.video_type.is_empty() {
            // Often used to skip a conversion
            warn!("{}: Empty video_type", x);
            continue;
        }
        if let Err(v) = Platform::from_str(&x.video_type) {
            error!("{}: {}", x, v);
            has_err = true;
        }
    }

    // Potential typo analysis
    info!("Checking potential typos...");
    for x in &check_result {
        if x.title.trim() != x.title {
            error!("{}: Spaces around title", x);
            has_err = true;
        }
        if x.artist.trim() != x.artist {
            error!("{}: Spaces around artist", x);
            has_err = true;
        }
    }

    // Unicode NFC check
    info!("Checking Unicode NFC conformity...");
    for x in &check_result {
        if !is_nfc(&x.title) {
            error!(
                "{}: Title is not in NFC, please change to '{}'",
                x,
                x.title.chars().nfc()
            );
            has_err = true;
        }
        if !is_nfc(&x.artist) {
            error!(
                "{}: Artist is not in NFC, please change to '{}'",
                x,
                x.artist.chars().nfc()
            );
            has_err = true;
        }
    }

    info!("Check similar metadatas...");

    // Title: ignore bracketed suffix
    let mut check_result_altered = check_result.clone();
    for i in check_result_altered.iter_mut() {
        i.title = RE.replace_all(&i.title, "").to_string();
    }

    similarity_check("Title", &check_result_altered, |x| &x.title);
    similarity_check("Artist", &check_result, |x| &x.artist);

    info!("Validating fields...");

    let converted_result = check_result
        .into_iter()
        .filter_map(|x| {
            let x_desc = x.to_string();
            let v: Result<Music> = x.try_into();
            match v {
                Ok(m) => Some(m),
                Err(e) => {
                    if &e.to_string() == "No status present" {
                        // Often used to skip a conversion
                        warn!("{}: Failed to convert to music: {}", x_desc, e);
                    } else {
                        error!("{}: Failed to convert to music: {}", x_desc, e);
                        has_err = true;
                    }
                    None
                }
            }
        })
        .collect::<Vec<Music>>();

    // Logic analysis
    info!("Checking entry logic...");
    for x in &converted_result {
        if let Err(v) = check_logic(x) {
            error!("{}: {}", x, v);
            has_err = true;
        }
    }

    info!("Check finished.");

    if opts.json_output {
        let base = serde_json::to_string(&converted_result).unwrap();
        println!("{}", base);
    }

    if has_err {
        Err(anyhow!("Some hard checks didn't pass."))
    } else {
        Ok(())
    }
}

#[test]
fn test_similarity_ratio() {
    // Normal cases
    assert_eq!(similarity_ratio("test", "test"), 1.0);
    assert_eq!(similarity_ratio("abcd", "efgh"), 0.0);
    // CJK
    assert_eq!(similarity_ratio("双海亚美", "双海真美"), 0.75);
    assert_eq!(similarity_ratio("中文Aka", "英文Aka"), 0.8);
}
