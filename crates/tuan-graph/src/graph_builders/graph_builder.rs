use crate::graph::Graph;

pub trait GraphBuilder {
    fn get_graph(&self) -> Graph;
}
