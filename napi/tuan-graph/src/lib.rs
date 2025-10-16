#![deny(clippy::all)]

use napi_derive::napi;
use tuan_graph::{cluster, graph};

#[napi(js_name = "Graph")]
pub struct Graph {
  pub(crate) inner: graph::Graph,
}

#[napi(object)]
pub struct GraphDescription {
  pub nodes: Vec<Node>,
  pub edges: Vec<Edge>,
}

#[napi]
impl Graph {
  #[napi]
  pub fn positioning(&mut self) {
    self.inner.positioning();
  }

  #[napi]
  pub fn describe(&self) -> GraphDescription {
    GraphDescription {
      nodes: self.nodes(),
      edges: self.edges(),
    }
  }

  #[napi]
  pub fn clusterize(&self, max_iters: u32) -> Vec<Cluster> {
    self
      .inner
      .clusterize(max_iters as usize)
      .into_iter()
      .map(Cluster::from_native)
      .collect()
  }

  #[napi(getter)]
  pub fn nodes(&self) -> Vec<Node> {
    self
      .inner
      .iter_nodes()
      .cloned()
      .map(Node::from_native)
      .collect()
  }

  #[napi(getter)]
  pub fn edges(&self) -> Vec<Edge> {
    self
      .inner
      .iter_edges()
      .cloned()
      .map(Edge::from_native)
      .collect()
  }
}

#[napi(object)]
pub struct Node {
  pub id: u32,
  pub label: String,
  pub file_path: String,
  pub position: (f64, f64),
}

impl Node {
  pub(crate) fn from_native(node: graph::Node) -> Self {
    Self {
      id: node.id as u32,
      label: node.label,
      file_path: node.file_path.to_string_lossy().to_string(),
      position: (node.position.0.into(), node.position.1.into()),
    }
  }
}

#[napi(object)]
pub struct Edge {
  pub from: u32,
  pub to: u32,
}

impl Edge {
  pub fn from_native(edge: graph::Edge) -> Self {
    Self {
      from: edge.from as u32,
      to: edge.to as u32,
    }
  }
}

#[napi(object)]
pub struct Cluster {
  pub id: u32,
  pub members: Vec<u32>,
}

impl Cluster {
  pub(crate) fn from_native(cluster: cluster::Cluster) -> Self {
    Self {
      id: cluster.id as u32,
      members: cluster.members.iter().map(|&id| id as u32).collect(),
    }
  }
}

#[napi]
pub mod typescript {
  use crate::Graph;
  use tuan_graph::graph_builders::{self, GraphBuilder as _};

  #[allow(dead_code)]
  #[napi]
  pub fn get_graph(project_path: String) -> Graph {
    let path = std::path::PathBuf::from(project_path);
    let builder = graph_builders::Typescript::new(path);
    let graph = builder.get_graph();
    Graph { inner: graph }
  }
}
