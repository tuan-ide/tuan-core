use crate::{graph::Node, graph_builders::typescript::visitor::Visitor};
use oxc_ast_visit::Visit;
use oxc_parser::{Parser, ParserReturn};
use oxc_resolver::{ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};
use oxc_span::SourceType;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub(super) struct Extractor {
    resolver: Resolver,
}

impl Extractor {
    pub(super) fn new(project_path: PathBuf) -> Self {
        Self {
            resolver: Self::build_resolver(&project_path),
        }
    }

    fn build_resolver(project_root: &Path) -> Resolver {
        let config_file = project_root.join("tsconfig.json");

        let options = ResolveOptions {
            extensions: vec![
                ".ts".into(),
                ".tsx".into(),
                ".mts".into(),
                ".cts".into(),
                ".js".into(),
                ".jsx".into(),
                ".mjs".into(),
                ".cjs".into(),
                ".json".into(),
                ".d.ts".into(),
            ],
            condition_names: vec!["node".into(), "import".into(), "types".into()],
            main_fields: vec![
                "types".into(),
                "typings".into(),
                "module".into(),
                "main".into(),
            ],
            tsconfig: Some(oxc_resolver::TsconfigDiscovery::Auto),
            ..ResolveOptions::default()
        };

        Resolver::new(options)
    }

    pub(super) fn find_typescript_files(&self, root: &Path) -> HashMap<PathBuf, Node> {
        WalkDir::new(root)
            .into_iter()
            .filter_entry(|entry| {
                !entry.path().components().any(|component| {
                    matches!(
                        component.as_os_str().to_str(),
                        Some("node_modules")
                            | Some(".git")
                            | Some("dist")
                            | Some("build")
                            | Some("coverage")
                            | Some(".svelte-kit")
                    )
                })
            })
            .filter_map(|res| {
                let e = res.ok()?;
                if e.file_type().is_file() {
                    match e.path().extension().and_then(|s| s.to_str()) {
                        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs") => {
                            let node = Node::from_path(e.into_path());
                            Some((node.file_path.clone(), node))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>()
    }

    pub(super) fn extract_typescript_imports(
        &self,
        node: &Node,
    ) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let source_code = std::fs::read_to_string(&node.file_path)?;
        let allocator = oxc_allocator::Allocator::default();
        let source_type = match node.file_path.extension().and_then(|s| s.to_str()) {
            Some("tsx") => SourceType::tsx(),
            Some("ts") => SourceType::ts(),
            Some("jsx") => SourceType::jsx(),
            Some("js") => SourceType::unambiguous().with_jsx(true).with_module(true),
            Some("mjs") => SourceType::mjs(),
            Some("cjs") => SourceType::cjs(),
            _ => SourceType::unambiguous(),
        };

        let ParserReturn {
            program, errors, ..
        } = Parser::new(&allocator, &source_code, source_type).parse();
        for e in &errors {
            eprintln!("Parse error in {}: {e}", node.file_path.display());
        }

        let mut visitor = Visitor::new(&node.file_path, &self.resolver);
        visitor.visit_program(&program);
        Ok(visitor.imports)
    }
}
