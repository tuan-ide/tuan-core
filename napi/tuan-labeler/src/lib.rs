#![deny(clippy::all)]

use std::collections::HashMap;

use napi_derive::napi;

#[napi]
pub fn label_files(
  project_files: Vec<String>,
  selected_files: Vec<String>,
  project_path: String,
) -> HashMap<String, f64> {
  tuan_labeler::label_files(
    project_files.iter().map(String::as_str).collect(),
    selected_files.iter().map(String::as_str).collect(),
    &project_path,
  )
  .unwrap()
}
