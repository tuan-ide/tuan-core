use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Graph {
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
        }
    }

    pub fn add_edge(&mut self, edge: Edge) {
        if !self
            .edges
            .iter()
            .any(|e| e.from == edge.from && e.to == edge.to)
        {
            self.edges.push(edge);
        }
    }
}

pub type NodeId = String;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub file_path: PathBuf,
    pub position: (f32, f32),
}

impl Node {
    pub fn from_path(path: PathBuf) -> Self {
        let id = path.to_string_lossy().to_string();
        let label = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| id.clone());
        Self {
            id,
            label,
            file_path: path,
            position: (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId) -> Self {
        Self { from, to }
    }
}
