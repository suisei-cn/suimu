use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FEResult<T: Serialize> {
  pub ok: bool,
  pub object: Option<T>,
  pub message: Option<String>,
}
