use crate::utils::check_csv;
use crate::maybemusic::MaybeMusic;

use anyhow::{Result, anyhow, ensure};
use serde::{Serialize, Deserialize, Serializer};
use std::path::PathBuf;
use std::fs::File;
use std::result::Result as StdResult;
use serde::ser::SerializeStruct;
use crate::compat::FEResult;

#[tauri::command]
pub fn get_maybemusic_by_csv_path(csv_path: String) -> FEResult<Vec<MaybeMusic>> {
    match __get_maybemusic_by_csv_path(csv_path) {
        Ok(v) => FEResult {
            ok: true,
            object: Some(v),
            message: None,
        },
        Err(e) => FEResult {
            ok: false,
            object: None,
            message: Some(e.to_string()),
        }
    }
}

fn __get_maybemusic_by_csv_path(csv_path: String) -> Result<Vec<MaybeMusic>> {
    let csv_file = PathBuf::from(csv_path);
    ensure!(csv_file.exists(), format!("{:?} does not exist", csv_file));

    let read_file = File::open(csv_file).unwrap();
    let check_result =
        check_csv(&read_file).map_err(|e| anyhow!(format!("CSV validation failed: {}", e)))?;

    Ok(check_result)
}
