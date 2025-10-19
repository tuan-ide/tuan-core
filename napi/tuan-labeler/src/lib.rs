#![deny(clippy::all)]

use std::{collections::HashMap, path::PathBuf};

use napi_derive::napi;

#[napi(js_name = "ProjectLabeler")]
pub struct ProjectLabeler {
  pub(crate) inner: tuan_labeler::ProjectLabeler,
}

#[napi]
impl ProjectLabeler {
  #[napi(constructor)]
  pub fn new(project_path: String, project_files: Vec<String>) -> Self {
    Self {
      inner: tuan_labeler::ProjectLabeler::new(
        project_path,
        project_files.iter().map(String::as_str).collect(),
      ),
    }
  }

  #[napi]
  pub fn label_files(&self, selected_files: Vec<String>) -> HashMap<String, f64> {
    self
      .inner
      .label_files(selected_files.iter().map(PathBuf::from).collect())
      .unwrap()
  }
}
