use ordered_float::OrderedFloat;
use serde::Serialize;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[repr(C)]
#[derive(Debug, Clone, Serialize)]
pub struct Graph {
    pub edges: HashSet<Edge>,
    pub nodes: HashMap<NodeId, Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: HashSet::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    pub fn positioning(&mut self) {
        let node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();

        let nodes_indexes: std::collections::HashMap<NodeId, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        let mut graph_layout = yifan_hu::Graph::new(node_ids.len());

        for edge in &self.edges {
            let from_idx = *nodes_indexes.get(&edge.from).unwrap();
            let to_idx = *nodes_indexes.get(&edge.to).unwrap();
            graph_layout.add_edge(from_idx, to_idx, 1.0);
        }

        let settings = yifan_hu::LayoutSettings {
            max_iterations: 10000,
            tolerance: 1e-3,
            ..Default::default()
        };

        let result = {
            measure_time::info_time!("Running graph layout algorithm");
            yifan_hu::multilevel_layout(&graph_layout, &settings)
        };

        for (idx, node_id) in node_ids.iter().enumerate() {
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.position = (
                    OrderedFloat(result.positions[idx].x),
                    OrderedFloat(result.positions[idx].y),
                );
            }
        }

        tracing::info!("Positioning completed in {} iterations", result.iterations);
    }
}

pub type NodeId = usize;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize)]
pub struct FileFingerprint {
    size: u64,
    modified_ns: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub file_path: PathBuf,
    pub position: (OrderedFloat<f64>, OrderedFloat<f64>),
    pub key: FileFingerprint,
}

impl Node {
    pub fn from_path(file_path: PathBuf) -> Option<Self> {
        static NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

        let file = File::open(&file_path).ok()?;
        let md = file.metadata().ok()?;
        let key = FileFingerprint {
            size: md.len(),
            modified_ns: md
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_nanos(),
        };
        drop(file);

        let label = file_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap();

        Some(Self {
            id,
            label,
            file_path,
            position: (OrderedFloat(0.0), OrderedFloat(0.0)),
            key,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId) -> Self {
        Self { from, to }
    }
}
