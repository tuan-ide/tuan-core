use std::path::PathBuf;

use ordered_float::OrderedFloat;

use crate::{ffi::Str, graph::NodeId};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub label: Str,
    pub file_path: Str,
    pub x: f64,
    pub y: f64,
}

impl Into<crate::graph::Node> for Node {
    fn into(self) -> crate::graph::Node {
        let file_path: String = self.file_path.into();
        let position = (OrderedFloat(self.x), OrderedFloat(self.y));

        crate::graph::Node {
            id: self.id,
            label: self.label.into(),
            file_path: PathBuf::from(file_path),
            position,
        }
    }
}

impl From<&crate::graph::Node> for Node {
    fn from(node: &crate::graph::Node) -> Self {
        Node {
            id: node.id,
            label: Str::from(node.label.clone()),
            file_path: Str::from(node.file_path.to_string_lossy().to_string()),
            x: node.position.0.into_inner(),
            y: node.position.1.into_inner(),
        }
    }
}
