#[macro_use]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FEResult<T: Serialize> {
  pub ok: bool,
  pub object: Option<T>,
  pub message: Option<String>,
}

impl<T: Serialize> From<anyhow::Result<T>> for FEResult<T> {
  fn from(r: anyhow::Result<T>) -> FEResult<T> {
    match r {
      Ok(v) => FEResult {
        ok: true,
        object: Some(v),
        message: None,
      },
      Err(e) => FEResult {
        ok: false,
        object: None,
        message: Some(e.to_string()),
      },
    }
  }
}

#[macro_export]
macro_rules! WrapCommand {
  ($src:ident ~= $dst:ident | $($nm:tt = $nn:ty),* | $ret:ty) => {
    #[tauri::command]
    pub fn $src($($nm: $nn)*) -> FEResult<$ret> {
      $dst($($nm: $nn)*).into()
    }
  };
}
