use crate::graph::Node;
use base64::prelude::*;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_resolver::Resolver;
use sourcemap::SourceMap;
use std::path::{Path, PathBuf};

pub(super) struct Visitor<'a> {
    pub(super) imports: Vec<Node>,
    current_file_dir: PathBuf,
    resolver: &'a Resolver,
}

impl<'a> Visitor<'a> {
    pub(super) fn new(current_file_path: &PathBuf, resolver: &'a Resolver) -> Self {
        Self {
            imports: Vec::new(),
            current_file_dir: current_file_path
                .parent()
                .unwrap_or(Path::new(""))
                .to_path_buf(),
            resolver,
        }
    }

    fn add_import<I>(&mut self, specifier: &str, identifiers: Option<I>)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let context = self.current_file_dir.clone();

        if let Ok(resolution) = self.resolver.resolve(context, specifier) {
            let path = resolution.full_path().to_path_buf();

            self.imports.push(Node::from_path(path.clone()));

            if let Some(identifiers) = identifiers
                && !(specifier.starts_with(".") || specifier.starts_with("/"))
                && !path.to_string_lossy().contains("node_modules")
            {
                if self
                    .fallback_import_with_sourcemap(&path, &identifiers.into_iter().collect())
                    .is_err()
                {
                    tracing::warn!(
                        "Failed to fallback imports with sourcemap for path: {:?}",
                        path
                    );
                }
            }
        }
    }

    fn fallback_import_with_sourcemap(
        &mut self,
        path: &PathBuf,
        identifiers: &Vec<&str>,
    ) -> Result<(), ()> {
        let smap = self
            .get_external_sourcemap(path)
            .or_else(|| self.get_inline_sourcemap(path))
            .ok_or(())?;

        for identifier in identifiers {
            for token in smap.tokens() {
                if let Some(name) = token.get_name() {
                    if name == *identifier {
                        if let Some(import_path) = token.get_source() {
                            let import_path = PathBuf::from(import_path);

                            let absolute_import_path = if import_path.is_absolute() {
                                import_path.clone()
                            } else {
                                path.parent()
                                    .unwrap_or(Path::new(""))
                                    .join(import_path.clone())
                                    .canonicalize()
                                    .unwrap_or(import_path.clone())
                            };

                            self.imports.push(Node::from_path(absolute_import_path));
                        }

                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_inline_sourcemap(&self, path: &PathBuf) -> Option<SourceMap> {
        let content = std::fs::read_to_string(path).ok()?;
        let lines: Vec<&str> = content.lines().collect();
        for line in lines.iter().rev() {
            if line.starts_with("//# sourceMappingURL=data:application/json;base64,") {
                let base64_data =
                    line.trim_start_matches("//# sourceMappingURL=data:application/json;base64,");
                if let Ok(decoded) = BASE64_STANDARD.decode(base64_data) {
                    if let Ok(smap) = SourceMap::from_slice(&decoded) {
                        return Some(smap);
                    }
                }
            }
        }
        None
    }

    fn get_external_sourcemap(&self, path: &PathBuf) -> Option<SourceMap> {
        let smap_path = path.with_extension("js.map");
        if smap_path.exists() {
            if let Ok(smap_content) = std::fs::read_to_string(smap_path) {
                if let Ok(smap) = SourceMap::from_slice(smap_content.as_bytes()) {
                    return Some(smap);
                }
            }
        }
        None
    }
}

impl<'a> Visit<'a> for Visitor<'a> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.add_import(
            decl.source.value.as_str(),
            decl.specifiers.as_ref().map(|specifiers| {
                specifiers.iter().map(|spec| match spec {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                        default.local.name.as_str()
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                        ns.local.name.as_str()
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(named) => named.local.name.as_str(),
                })
            }),
        );
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.add_import(
            decl.source.value.as_str(),
            Some(decl.exported.as_ref().map_or(vec![], |e| match e {
                ModuleExportName::IdentifierName(identifier_name) => {
                    vec![identifier_name.name.as_str()]
                }
                ModuleExportName::IdentifierReference(identifier_reference) => {
                    vec![identifier_reference.name.as_str()]
                }
                ModuleExportName::StringLiteral(string_literal) => {
                    vec![string_literal.value.as_str()]
                }
            })),
        );
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.add_import(source.value.as_str(), None::<Vec<&str>>);
        }
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::ImportExpression(_) = &expr.callee {
            if let Some(first_arg) = expr.arguments.first() {
                if let Argument::StringLiteral(str_lit) = first_arg {
                    self.add_import(str_lit.value.as_str(), None::<Vec<&str>>);
                }
            }
        }

        if let Expression::Identifier(ident) = &expr.callee {
            if ident.name == "require" {
                if let Some(first_arg) = expr.arguments.first() {
                    if let Argument::StringLiteral(str_lit) = first_arg {
                        self.add_import(str_lit.value.as_str(), None::<Vec<&str>>);
                    }
                }
            }
        }

        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            self.visit_argument(arg);
        }
    }

    fn visit_member_expression(&mut self, expr: &MemberExpression<'a>) {
        if let MemberExpression::StaticMemberExpression(static_expr) = expr {
            if let Expression::CallExpression(call_expr) = &static_expr.object {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    if ident.name == "require" {
                        if let Some(first_arg) = call_expr.arguments.first() {
                            if let Argument::StringLiteral(str_lit) = first_arg {
                                self.add_import(str_lit.value.as_str(), None::<Vec<&str>>);
                            }
                        }
                    }
                }
            }
        }

        match expr {
            MemberExpression::ComputedMemberExpression(computed) => {
                self.visit_expression(&computed.object);
                self.visit_expression(&computed.expression);
            }
            MemberExpression::StaticMemberExpression(static_expr) => {
                self.visit_expression(&static_expr.object);
            }
            MemberExpression::PrivateFieldExpression(private) => {
                self.visit_expression(&private.object);
            }
        }
    }
}
