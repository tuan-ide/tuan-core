use crate::{
    graph::{Edge, Graph, Node},
    graph_builders::GraphBuilder,
};
use std::path::PathBuf;
use rayon::prelude::*;

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
        let mut graph = Graph::new();

        let root = &project_path;
        let extractor = extractor::Extractor::new(project_path.clone());
        let ts_files = {
            measure_time::info_time!("Finding TypeScript files");
            extractor.find_typescript_files(root)
        };

        let results: Vec<(Node, Vec<Edge>)> = {
            measure_time::info_time!("Extracting imports from TypeScript files");
            ts_files
                .par_iter()
                .map(|file| {
                    let mut edges = Vec::new();
                    match extractor.extract_typescript_imports(file) {
                        Ok(imports) => {
                            for imported_file in imports {
                                edges.push(Edge::new(file.id.clone(), imported_file.id.clone()));
                            }
                        }
                        Err(e) => tracing::error!(
                            "Error extracting imports from {}: {}",
                            file.file_path.display(),
                            e
                        ),
                    }
                    (file.clone(), edges)
                })
                .collect()
        };

        {
            measure_time::info_time!("Inserting nodes and edges into graph");
            for (file, edges) in results {
                graph.add_node(file);
                for edge in edges {
                    graph.add_edge(edge);
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use tracing::info;

    #[test]
    fn it_builds_a_graph() {
        let (fixture_dir, temp_dir) =
            setup_test_project("https://github.com/webpack/webpack", "4fabb75");

        let graph = {
            measure_time::info_time!("Building graph for webpack");

            let typescript = {
                measure_time::info_time!("Initializing Typescript graph builder");
                Typescript::new(fixture_dir)
            };

            {
                measure_time::info_time!("Getting graph from Typescript graph builder");
                typescript.get_graph()
            }
        };

        assert!(graph.nodes.len() > 0);
        assert!(graph.edges.len() > 0);

        info!("Graph has {} nodes", graph.nodes.len());
        info!("Graph has {} edges", graph.edges.len());

        drop(temp_dir);
    }

    fn setup_test_project(git_repo: &str, commit: &str) -> (PathBuf, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let fixture_dir = temp_dir.path();
        std::fs::create_dir_all(&temp_dir).unwrap();

        info!("Cloning {} into {}", git_repo, fixture_dir.display());
        let output = std::process::Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                git_repo,
                &fixture_dir.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert!(output.status.success());

        info!("Checking out commit {}", commit);
        let output = std::process::Command::new("git")
            .args(&["-C", &fixture_dir.to_string_lossy(), "checkout", commit])
            .output()
            .unwrap();
        assert!(output.status.success());

        (fixture_dir.to_path_buf(), temp_dir)
    }
}
