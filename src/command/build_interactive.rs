use std::fs::read_dir;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use log::debug;
use requestty::question::Completions;
use requestty::{Answer, Answers, Question};

use crate::utils::{FromInteractive, IsHidden, Prefix};
use crate::{build, get_answer, BuildOpt};

fn file_exists(raw_path: &str, _prev: &Answers) -> Result<(), String> {
    let path = PathBuf::from(raw_path);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{} does not exist or is not a file", raw_path))
    }
}

fn is_advance_mode(prev: &Answers) -> bool {
    match prev.get("advance") {
        Some(Answer::Bool(advance_opt)) => advance_opt.to_owned(),
        _ => false,
    }
}

fn get_all_visible_children(path: &Path) -> Result<Vec<PathBuf>> {
    Ok(read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.is_hidden())
        .map(|entry| entry.path())
        .collect())
}

fn format_paths<T: Iterator<Item = PathBuf>>(paths: T) -> Vec<String> {
    let mut ret = paths
        .filter_map(|path| {
            path.to_str().map(|x| {
                if path.is_dir() {
                    x.to_owned() + "/"
                } else {
                    x.to_owned()
                }
            })
        })
        .collect::<Vec<_>>();
    ret.sort();
    ret
}

fn get_path(mut path: PathBuf) -> Result<Vec<String>> {
    if !path.is_absolute() {
        path = path.canonicalize()?
    }
    if path.is_dir() {
        let entries = get_all_visible_children(&path)?;
        Ok(format_paths(entries.into_iter()))
    } else if let Some(parent) = path.parent() {
        let children = get_all_visible_children(parent)?.into_iter().filter(|x| {
            match (x.file_name(), path.file_name()) {
                (Some(cur_name), Some(input_name)) => cur_name.starts_with(input_name),
                (Some(_), None) => true,
                _ => false,
            }
        });
        Ok(format_paths(children))
    } else {
        bail!("{} does not have parent", path.display())
    }
}

fn autocomplete_path(current: String, _ans: &Answers) -> Completions<String> {
    match get_path(PathBuf::from(if current.is_empty() {
        "./".to_owned()
    } else {
        current.clone()
    })) {
        Ok(ret) if !ret.is_empty() => ret.into(),
        _ => Completions::from([current]),
    }
}

impl FromInteractive for BuildOpt {
    fn from_interactive() -> Result<Self> {
        let csv_file_question = Question::input("csv_file")
            .message("Path to CSV file")
            .default("./suisei-music.csv")
            .auto_complete(autocomplete_path)
            .validate(file_exists)
            .build();
        let out_dir_question = Question::input("output_dir")
            .message("Output path")
            .auto_complete(autocomplete_path)
            .build();
        let src_dir_question = Question::input("source_dir")
            .message("Source path")
            .auto_complete(autocomplete_path)
            .build();
        let extra_setting_question = Question::confirm("advance")
            .message("Advanced settings")
            .default(false)
            .build();
        let dry_run_question = Question::confirm("dry_run")
            .when(is_advance_mode)
            .message("Run without actually processing musics")
            .default(false)
            .build();
        let ffmpeg_question = Question::input("ffmpeg")
            .when(is_advance_mode)
            .message("ffpmeg executable")
            .default("ffmpeg")
            .build();
        let ytdl_question = Question::input("ytdl")
            .when(is_advance_mode)
            .message("youtube-dl executable")
            .default("youtube-dl")
            .build();
        let output_json_question = Question::input("output_json")
            .when(is_advance_mode)
            .message("output json path")
            .default("")
            .build();
        let baseurl_question = Question::input("baseurl")
            .when(is_advance_mode)
            .message("output json URL base")
            .default("")
            .build();
        let output_diff_question = Question::input("output_diff")
            .when(is_advance_mode)
            .message("output diff json path")
            .default("")
            .build();
        let answers = requestty::prompt([
            csv_file_question,
            out_dir_question,
            src_dir_question,
            extra_setting_question,
            dry_run_question,
            ffmpeg_question,
            ytdl_question,
            output_json_question,
            baseurl_question,
            output_diff_question,
        ])?;
        debug!("Answers: {:#?}", answers);

        let opts = Self::new(
            get_answer!(answers, "csv_file").into(),
            get_answer!(answers, "output_dir").into(),
            get_answer!(answers, "source_dir").into(),
            get_answer!(answers, as_bool, "dry_run"),
            get_answer!(answers, "ffmpeg"),
            get_answer!(answers, "ytdl"),
            match get_answer!(answers, "output_json").as_ref() {
                "" => None,
                i => Some(i.into()),
            },
            match get_answer!(answers, "baseurl").as_ref() {
                "" => None,
                i => Some(i.into()),
            },
            match get_answer!(answers, "output_diff").as_ref() {
                "" => None,
                i => Some(i.into()),
            },
        );

        Ok(opts)
    }
}

pub fn build_interactive() -> Result<()> {
    let opts = BuildOpt::from_interactive()?;
    build(opts)?;
    Ok(())
}

#[test]
#[ignore = "Need human input"]
fn test_autocomplete() {
    requestty::prompt_one(
        Question::input("test")
            .message("Dir")
            .auto_complete(autocomplete_path)
            .build(),
    )
    .unwrap();
}
