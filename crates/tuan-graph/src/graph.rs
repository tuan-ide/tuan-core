pub struct Graph {
  pub edges: Vec<Edge>,
  pub nodes: Vec<Node>,
}

pub type NodeId = String;

pub struct Node {
  pub id: NodeId,
  pub label: String,
  pub position: (f32, f32),
}

pub struct Edge {
  pub from: NodeId,
  pub to: NodeId,
}
