use crate::{
    graph::{Edge, Graph, Node},
    graph_builders::GraphBuilder,
};
use path_clean::PathClean;
use rayon::prelude::*;
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
        let mut graph = Graph::new();

        let project_path = project_path.clean();
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
                .map(|(_, file)| {
                    let mut edges = Vec::new();
                    match extractor.extract_typescript_imports(file) {
                        Ok(imports) => {
                            for imported_file in imports {
                                // TODO: support imports with ? (like import x from 'y?type=script')
                                if let Some(import_node) = ts_files.get(&imported_file.file_path) {
                                    edges.push(Edge::new(file.id.clone(), import_node.id.clone()));
                                }
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
            for (file, _edges) in &results {
                graph.add_node(file.clone());
            }
            for (_file, edges) in &results {
                for edge in edges {
                    graph.add_edge(edge.clone());
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
        let (fixture_dir, temp_dir) = setup_test_project(
            "https://github.com/webpack/webpack",
            "7fc28f1e53634e1c6f082713ded2e5b3f3a96585",
        );

        let mut graph = {
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

        info!("Graph has {} nodes", graph.iter_nodes().count());
        info!("Graph has {} edges", graph.iter_edges().count());

        assert!(graph.iter_nodes().count() == 7745);
        assert!(graph.iter_edges().count() == 5389);

        drop(temp_dir);

        {
            measure_time::info_time!("Positioning graph");
            graph.positioning();
        }
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
        assert!(
            output.status.success(),
            "`git clone --depth 1 {} {}` failed: {:?}",
            git_repo,
            fixture_dir.display(),
            output
        );

        info!("Checking out commit {}", commit);
        let output = std::process::Command::new("git")
            .args(&["-C", &fixture_dir.to_string_lossy(), "checkout", commit])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "`git -C {} checkout {}` failed: {:?}",
            fixture_dir.display(),
            commit,
            output
        );

        (fixture_dir.to_path_buf(), temp_dir)
    }
}
