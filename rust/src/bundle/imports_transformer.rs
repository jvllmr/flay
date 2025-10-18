use std::collections::HashSet;

use crate::common::ast::checkers::{is_dynamic_import_mut, is_importlib_import};
use crate::common::ast::generate_source;
use crate::common::ast::transformer::{
    Transformer, walk_annotation, walk_body, walk_expr, walk_stmt,
};
use crate::common::module_spec::is_in_std_lib;
use pyo3::pyfunction;

use ruff_python_ast::name::Name;
use ruff_python_ast::{
    Alias, AtomicNodeIndex, Expr, ExprAttribute, ExprContext, ExprName, Identifier, Stmt,
    StmtImport, StringLiteral, StringLiteralFlags, StringLiteralValue,
};
use ruff_python_parser::parse_module;
use ruff_text_size::TextRange;

struct ImportsTransformer {
    top_level_package: String,
    vendor_module_name: String,
    names_to_sanitize: HashSet<String>,
    needed_imports: Vec<HashSet<String>>,
    is_in_annotation: bool,
    importlib_package_alias: Option<String>,
}

impl ImportsTransformer {
    fn new(top_level_package: String, vendor_module_name: String) -> Self {
        ImportsTransformer {
            top_level_package,
            vendor_module_name,
            names_to_sanitize: HashSet::new(),
            needed_imports: Vec::new(),
            is_in_annotation: false,
            importlib_package_alias: None,
        }
    }

    fn get_vendor_string(&self) -> String {
        return self.top_level_package.to_owned() + "." + &self.vendor_module_name;
    }

    fn _prepend_vendor(&self, node: &Name) -> Name {
        let node_str = node.as_str();
        return Name::new(self.get_vendor_string() + "." + node_str);
    }

    fn _prepend_vendor_import<'a>(&mut self, node: Identifier, module_spec: &str) -> Identifier {
        if module_spec.starts_with(&self.top_level_package) || is_in_std_lib(module_spec) {
            return node.clone();
        }

        return Identifier::new(
            self._prepend_vendor(&Name::new(node.id)).to_owned(),
            node.range,
        );
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
        if let Some(importlib_package_alias) = is_importlib_import(&stmt) {
            self.importlib_package_alias = Some(importlib_package_alias);
        }

        match stmt {
            Stmt::Import(mut import) => {
                import.names = import
                    .names
                    .iter()
                    .map(|name| Alias {
                        range: name.range,
                        name: self._prepend_vendor_import(name.name.to_owned(), name.name.as_str()),
                        asname: Some(self.decide_asname(&name.name, &name.asname)),
                        node_index: AtomicNodeIndex::default(),
                    })
                    .collect();
                Some(Stmt::Import(import))
            }
            Stmt::ImportFrom(mut import_from) => {
                if import_from.level == 0 {
                    if let Some(module_node) = &import_from.module {
                        let module_spec = module_node.as_str();
                        let new_module =
                            self._prepend_vendor_import(module_node.to_owned(), module_spec);
                        import_from.module = Option::from(new_module);
                    }
                }
                Some(Stmt::ImportFrom(import_from))
            }
            _ => walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, mut expr: Expr) -> Option<ruff_python_ast::Expr> {
        if let Some(dynamic_import_expr) =
            is_dynamic_import_mut(&mut expr, self.importlib_package_alias.as_ref())
        {
            match dynamic_import_expr {
                Expr::StringLiteral(literal) => {
                    let old_value = literal.value.to_str();
                    if !old_value.starts_with(&self.top_level_package) {
                        literal.value = StringLiteralValue::single(StringLiteral {
                            range: TextRange::default(),
                            node_index: AtomicNodeIndex::default(),
                            value: format!("{}.{}", self.get_vendor_string(), old_value)
                                .into_boxed_str(),
                            flags: StringLiteralFlags::empty(),
                        })
                    }
                }
                _ => {}
            }
        }
        match expr {
            Expr::Attribute(attribute) => {
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
                            self.visit_expr(value_expr);
                            return Some(Expr::Attribute(attribute));
                        }
                    }
                }

                for i in 1..name_parts.len() {
                    let test_name = &name_parts[0..i].join(".");
                    if self.names_to_sanitize.contains(test_name) {
                        let mut target_attribute = attribute_parts[i - 1].clone();

                        target_attribute.value = Box::new(Expr::Name(ExprName {
                            range: target_attribute.range,
                            id: Name::new(test_name.replace(".", "_")),
                            node_index: AtomicNodeIndex::default(),
                            ctx: expr_context,
                        }));

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
pub fn transform_imports(
    source: &str,
    top_level_package: &str,
    vendor_module_name: &str,
) -> String {
    let parsed = parse_module(source).unwrap();
    let module = parsed.syntax();

    let mut transformer = ImportsTransformer::new(
        top_level_package.to_string(),
        vendor_module_name.to_string(),
    );
    let new_body = transformer.visit_body(&module.body);

    return generate_source(&new_body, parsed, source);
}
