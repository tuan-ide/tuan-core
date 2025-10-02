#![deny(clippy::all)]

use napi_derive::napi;
use tuan_graph::graph::{Edge, Graph, Node};

#[napi(js_name = "Graph")]
pub struct NapiGraph {
  pub(crate) inner: Graph,
}

#[napi]
impl NapiGraph {
  #[napi]
  pub fn positioning(&mut self) {
    self.inner.positioning();
  }

  #[napi(getter)]
  pub fn nodes(&self) -> Vec<NapiNode> {
    self
      .inner
      .iter_nodes()
      .cloned()
      .map(NapiNode::from_native)
      .collect()
  }

  #[napi(getter)]
  pub fn edges(&self) -> Vec<NapiEdge> {
    self
      .inner
      .iter_edges()
      .cloned()
      .map(NapiEdge::from_native)
      .collect()
  }
}

#[napi(js_name = "Node")]
pub struct NapiNode {
  pub(crate) inner: Node,
}

impl NapiNode {
  pub fn from_native(node: Node) -> Self {
    Self { inner: node }
  }
}

#[napi]
impl NapiNode {
  #[napi(getter)]
  pub fn id(&self) -> usize {
    self.inner.id
  }

  #[napi(getter)]
  pub fn label(&self) -> String {
    self.inner.label.clone()
  }

  #[napi(getter)]
  pub fn file_path(&self) -> String {
    self.inner.file_path.to_string_lossy().to_string()
  }

  #[napi(getter)]
  pub fn position(&self) -> (f64, f64) {
    (self.inner.position.0.into(), self.inner.position.1.into())
  }
}

#[napi(js_name = "Edge")]
pub struct NapiEdge {
  pub(crate) inner: Edge,
}

impl NapiEdge {
  pub fn from_native(edge: Edge) -> Self {
    Self { inner: edge }
  }
}

#[napi]
impl NapiEdge {
  #[napi(getter)]
  pub fn from(&self) -> usize {
    self.inner.from
  }

  #[napi(getter)]
  pub fn to(&self) -> usize {
    self.inner.to
  }
}

#[napi]
pub mod typescript {
  use crate::NapiGraph;
  use tuan_graph::graph_builders::{self, GraphBuilder as _};

  #[allow(dead_code)]
  #[napi]
  pub fn get_graph(project_path: String) -> NapiGraph {
    let path = std::path::PathBuf::from(project_path);
    let builder = graph_builders::Typescript::new(path);
    let graph = builder.get_graph();
    NapiGraph { inner: graph }
  }
}
