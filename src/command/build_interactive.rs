use std::path::PathBuf;

use anyhow::Result;
use log::debug;
use requestty::{Answer, Answers, Question};

use crate::utils::FromInteractive;

use crate::{build, get_answer, BuildOpt};

fn path_exists(path: &str, _prev: &Answers) -> Result<(), String> {
    if PathBuf::from(path).exists() {
        Ok(())
    } else {
        Err(format!("{} does not exist", path))
    }
}

fn is_advance_mode(prev: &Answers) -> bool {
    match prev.get("advance") {
        Some(Answer::Bool(advance_opt)) => advance_opt.to_owned(),
        _ => false,
    }
}

impl FromInteractive for BuildOpt {
    fn from_interactive() -> Result<Self> {
        let csv_file_question = Question::input("csv_file")
            .message("Path to CSV file")
            .default("./suisei-music.csv")
            .validate(path_exists)
            .build();
        let out_dir_question = Question::input("output_dir").message("Output path").build();
        let src_dir_question = Question::input("source_dir").message("Source path").build();
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
        let answers = requestty::prompt([
            csv_file_question,
            out_dir_question,
            src_dir_question,
            extra_setting_question,
            dry_run_question,
            ffmpeg_question,
            ytdl_question,
        ])?;
        debug!("Answers: {:#?}", answers);

        let opts = Self::new(
            get_answer!(answers, "csv_file").into(),
            get_answer!(answers, "output_dir").into(),
            get_answer!(answers, "source_dir").into(),
            get_answer!(answers, as_bool, "dry_run"),
            get_answer!(answers, "ffmpeg"),
            get_answer!(answers, "ytdl"),
        );

        Ok(opts)
    }
}

pub fn build_interactive() -> Result<()> {
    let opts = BuildOpt::from_interactive()?;
    build(opts)?;
    Ok(())
}
