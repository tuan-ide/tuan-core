use crate::{graph::NodeId};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

impl Into<crate::graph::Edge> for Edge {
    fn into(self) -> crate::graph::Edge {
        crate::graph::Edge {
            from: self.from,
            to: self.to,
        }
    }
}

impl From<&crate::graph::Edge> for Edge {
    fn from(edge: &crate::graph::Edge) -> Self {
        Edge {
            from: edge.from,
            to: edge.to,
        }
    }
}
