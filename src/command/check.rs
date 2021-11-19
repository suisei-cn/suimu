use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, ensure, Result};
use lazy_static::lazy_static;
use levenshtein::levenshtein;
use log::{info, warn};
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
        }
        if x.artist.trim() != x.artist {
            warn!("{}: Spaces around artist", x);
        }
    }

    // Unicode NFC check
    info!("Checking Unicode NFC conformity...");
    for x in &check_result {
        if !is_nfc(&x.title) {
            warn!(
                "{}: Title is not in NFC, please change to '{}'",
                x,
                x.title.chars().nfc()
            );
        }
        if !is_nfc(&x.artist) {
            warn!(
                "{}: Artist is not in NFC, please change to '{}'",
                x,
                x.artist.chars().nfc()
            );
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

    let converted_result = check_result
        .into_iter()
        .map(|x| {
            let x_desc = x.to_string();
            let v: Result<Music> = x.try_into();
            match v {
                Ok(m) => Some(m),
                Err(e) => {
                    warn!("{}: Failed to convert to music: {}", x_desc, e);
                    None
                }
            }
        })
        .flatten()
        .collect::<Vec<Music>>();

    // Logic analysis
    info!("Checking entry logic...");
    for x in &converted_result {
        if let Err(v) = check_logic(x) {
            warn!("{}: {}", x, v);
        }
    }

    info!("Check finished.");

    if opts.json_output {
        let base = serde_json::to_string(&converted_result).unwrap();
        println!("{}", base);
    }

    Ok(())
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
