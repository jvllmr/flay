use std::collections::HashSet;

use crate::common::ast::generate_source;
use crate::common::ast::transformer::{
    Transformer, walk_annotation, walk_body, walk_expr, walk_stmt,
};
use pyo3::pyfunction;

use ruff_python_ast::name::Name;
use ruff_python_ast::{
    Alias, AtomicNodeIndex, Expr, ExprAttribute, ExprContext, ExprName, Identifier, Stmt,
    StmtImport, StringLiteral,
};
use ruff_python_parser::parse_module;
use ruff_text_size::TextRange;

struct ImportsTransformer {
    names_to_sanitize: HashSet<String>,
    needed_imports: Vec<HashSet<String>>,
    is_in_annotation: bool,
}

impl ImportsTransformer {
    fn new() -> Self {
        ImportsTransformer {
            names_to_sanitize: HashSet::new(),
            needed_imports: Vec::new(),
            is_in_annotation: false,
        }
    }

    fn decide_asname(&mut self, name: &Identifier, asname: &Option<Identifier>) -> Identifier {
        match asname {
            Some(asname_value) => asname_value.to_owned(),
            None => {
                if name.contains(".") {
                    self.names_to_sanitize.insert(name.to_string());

                    let name_parts: Vec<&str> = name.split(".").collect();
                    let name_parts_len = name_parts.len();
                    for i in 1..name_parts_len {
                        if i != name_parts_len {
                            self.needed_imports[0].insert(name_parts[0..i].join("."));
                        }
                    }
                    Identifier::new(name.replace(".", "_"), name.range)
                } else {
                    name.to_owned()
                }
            }
        }
    }
}

impl Transformer for ImportsTransformer {
    fn visit_annotation(&mut self, expr: Expr) -> Option<Expr> {
        self.is_in_annotation = true;
        let res = walk_annotation(self, expr);
        self.is_in_annotation = false;
        res
    }
    fn visit_string_literal(
        &mut self,
        mut string_literal: ruff_python_ast::StringLiteral,
    ) -> Option<StringLiteral> {
        if self.is_in_annotation && string_literal.value.contains(".") {
            for sanitize_name in &self.names_to_sanitize {
                if string_literal.value.contains(sanitize_name) {
                    string_literal.value = string_literal
                        .value
                        .replace(sanitize_name, &sanitize_name.replace(".", "_"))
                        .into_boxed_str()
                }
            }
        }

        Some(string_literal)
    }

    fn visit_body(&mut self, body: &[Stmt]) -> Vec<Stmt> {
        self.needed_imports.insert(0, HashSet::new());
        let mut new_body = walk_body(self, body);
        let mut current_needed_imports = self.needed_imports[0].to_owned();
        self.needed_imports.remove(0);
        let mut has_future_import = false;
        for stmt in &new_body {
            match stmt {
                Stmt::Import(import) => {
                    for name in &import.names {
                        if current_needed_imports.contains(name.name.as_str()) {
                            current_needed_imports.remove(name.name.as_str());
                        }
                    }
                }
                Stmt::ImportFrom(import_from) => {
                    if import_from
                        .module
                        .as_ref()
                        .is_some_and(|x| x == "__future__")
                    {
                        has_future_import = true;
                    }
                }
                _ => {}
            }
        }
        let insertion_index = match has_future_import {
            false => 0,
            true => 2,
        };
        for needed_import in &current_needed_imports {
            new_body.insert(
                insertion_index,
                Stmt::Import(StmtImport {
                    names: vec![Alias {
                        range: TextRange::default(),
                        asname: Some(Identifier::new(
                            needed_import.replace(".", "_"),
                            TextRange::default(),
                        )),
                        name: Identifier::new(needed_import, TextRange::default()),
                        node_index: AtomicNodeIndex::default(),
                    }],
                    node_index: AtomicNodeIndex::default(),
                    range: TextRange::default(),
                }),
            );
        }

        return new_body;
    }

    fn visit_stmt(&mut self, stmt: ruff_python_ast::Stmt) -> Option<ruff_python_ast::Stmt> {
        match stmt {
            Stmt::Import(mut import) => {
                import.names = import
                    .names
                    .iter()
                    .map(|name| Alias {
                        range: name.range,
                        name: name.name.to_owned(),
                        asname: Some(self.decide_asname(&name.name, &name.asname)),
                        node_index: AtomicNodeIndex::default(),
                    })
                    .collect();
                Some(Stmt::Import(import))
            }
            _ => walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, expr: Expr) -> Option<ruff_python_ast::Expr> {
        match expr {
            Expr::Attribute(mut attribute) => {
                let mut name_parts: Vec<String> = vec![attribute.attr.to_string()];

                let mut attribute_parts: Vec<ExprAttribute> = vec![attribute.clone()];
                #[allow(unused_assignments)]
                let mut expr_context = ExprContext::Load;
                loop {
                    let value_expr = *attribute_parts[0].value.clone();

                    match value_expr {
                        Expr::Attribute(attr) => {
                            name_parts.insert(0, attr.attr.to_string());
                            attribute_parts.insert(0, attr);
                        }
                        Expr::Name(name) => {
                            name_parts.insert(0, name.id.to_string());
                            expr_context = name.ctx;
                            break;
                        }
                        _ => {
                            if let Some(new_value) = self.visit_expr(value_expr) {
                                attribute.value = Box::new(new_value);
                            }
                            break;
                        }
                    }
                }

                for i in 1..name_parts.len() {
                    let test_name = &name_parts[0..i].join(".");
                    if self.names_to_sanitize.contains(test_name) {
                        let rest = attribute_parts.len() - i;
                        let mut target_attribute = attribute_parts[i - 1 + rest].to_owned();

                        let mut new_name = test_name.replace(".", "_");

                        if rest > 0 && i + rest < name_parts.len() {
                            new_name =
                                format!("{}.{}", new_name, name_parts[i..i + rest].join("."));
                        }

                        target_attribute.value = Box::new(Expr::Name(ExprName {
                            range: target_attribute.range,
                            id: Name::new(new_name),
                            node_index: AtomicNodeIndex::default(),
                            ctx: expr_context,
                        }));
                        println!("{:?}", target_attribute);
                        return Some(Expr::Attribute(target_attribute));
                    }
                }

                Some(Expr::Attribute(attribute))
            }
            _ => walk_expr(self, expr),
        }
    }
}

#[pyfunction]
pub fn transform_imports(source: &str) -> String {
    let parsed = parse_module(source).unwrap();
    let module = parsed.syntax();

    let mut transformer = ImportsTransformer::new();
    let new_body = transformer.visit_body(&module.body);

    return generate_source(&new_body, parsed, source);
}
