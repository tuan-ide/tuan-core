use crate::graph::Node;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_resolver::Resolver;
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

    fn add_import(&mut self, specifier: &str) {
        let context = self.current_file_dir.clone();

        if let Ok(resolution) = self.resolver.resolve(context, specifier) {
            let path = resolution.full_path().to_path_buf();
            self.imports.push(Node::from_path(path));
        }
    }
}

impl<'a> Visit<'a> for Visitor<'a> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.add_import(decl.source.value.as_str());
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.add_import(decl.source.value.as_str());
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.add_import(source.value.as_str());
        }
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::ImportExpression(_) = &expr.callee {
            if let Some(first_arg) = expr.arguments.first() {
                if let Argument::StringLiteral(str_lit) = first_arg {
                    self.add_import(str_lit.value.as_str());
                }
            }
        }

        if let Expression::Identifier(ident) = &expr.callee {
            if ident.name == "require" {
                if let Some(first_arg) = expr.arguments.first() {
                    if let Argument::StringLiteral(str_lit) = first_arg {
                        self.add_import(str_lit.value.as_str());
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
                                self.add_import(str_lit.value.as_str());
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
