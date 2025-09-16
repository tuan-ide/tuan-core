use std::{collections::HashSet, path::PathBuf};

use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct Graph {
    pub edges: HashSet<Edge>,
    pub nodes: HashSet<Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: HashSet::new(),
            nodes: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }
}

pub type NodeId = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub file_path: PathBuf,
    pub position: (OrderedFloat<f32>, OrderedFloat<f32>),
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
            position: (OrderedFloat(0.0), OrderedFloat(0.0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId) -> Self {
        Self { from, to }
    }
}
