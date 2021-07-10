use crate::{maybemusic::MaybeMusic, utils::check_csv};

use crate::WrapCommand;
use anyhow::{anyhow, ensure, Result};
use std::{fs::File, path::PathBuf};

WrapCommand!(get_maybemusic_by_csv_path ~= __get_maybemusic_by_csv_path | csv_path = String | Vec<MaybeMusic>);

fn __get_maybemusic_by_csv_path(csv_path: String) -> Result<Vec<MaybeMusic>> {
  let csv_file = PathBuf::from(csv_path);
  ensure!(csv_file.exists(), format!("{:?} does not exist", csv_file));

  let read_file = File::open(csv_file).unwrap();
  let check_result =
    check_csv(&read_file).map_err(|e| anyhow!(format!("CSV validation failed: {}", e)))?;

  Ok(check_result)
}
