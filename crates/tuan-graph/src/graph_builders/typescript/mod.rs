use crate::{
    graph::{Edge, Graph},
    graph_builders::GraphBuilder,
};
use std::path::PathBuf;

mod extractor;
mod visitor;

pub struct Typescript {
    graph: Graph,
}

impl GraphBuilder for Typescript {
    fn get_graph(&self) -> Graph {
        self.graph.clone()
    }
}

impl Typescript {
    pub fn new(project_path: PathBuf) -> Self {
        let graph = Self::create_graph(project_path);
        Self { graph }
    }

    fn create_graph(project_path: PathBuf) -> Graph {
        let root = &project_path;
        let extractor = extractor::Extractor::new(project_path.clone());
        let ts_files = extractor.find_typescript_files(root);
        let mut graph = Graph::new();

        for file in &ts_files {
            graph.add_node(file.clone());

            match extractor.extract_typescript_imports(file) {
                Ok(imports) => {
                    for imported_file in imports {
                        graph.add_edge(Edge::new(file.id.clone(), imported_file.id.clone()));
                    }
                }
                Err(e) => tracing::error!(
                    "Error extracting imports from {}: {}",
                    file.file_path.display(),
                    e
                ),
            }
        }

        graph
    }
}
