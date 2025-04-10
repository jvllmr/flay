use crate::common::ast::generate_source;
use crate::common::ast::transformer::{Transformer, walk_annotation, walk_expr, walk_stmt};
use crate::common::module_spec::{get_top_level_package, is_in_std_lib};
use pyo3::pyfunction;

use ruff_python_ast::name::Name;
use ruff_python_ast::{Alias, Expr, ExprName, Identifier, Stmt, StringLiteral};
use ruff_python_parser::parse_module;

use std::collections::HashSet;
struct ImportsTransformer {
    top_level_package: String,
    vendor_module_name: String,
    affected_names: HashSet<String>,
    is_in_annotation: bool,
}

impl ImportsTransformer {
    fn new(top_level_package: String, vendor_module_name: String) -> Self {
        ImportsTransformer {
            top_level_package,
            vendor_module_name,
            affected_names: HashSet::new(),
            is_in_annotation: false,
        }
    }

    fn get_vendor_string(&self) -> String {
        return self.top_level_package.to_owned() + "." + &self.vendor_module_name;
    }

    fn _prepend_vendor(&self, node: &Name) -> Name {
        let node_str = node.as_str();
        return Name::new(self.get_vendor_string() + "." + node_str);
    }

    fn _prepend_vendor_import<'a>(
        &mut self,
        node: Identifier,
        module_spec: &str,
        references_need_update: bool,
    ) -> Identifier {
        if module_spec.starts_with(&self.top_level_package)
            || is_in_std_lib(get_top_level_package(module_spec))
        {
            return node.clone();
        }
        if references_need_update {
            self.affected_names.insert(node.to_string());
        }
        return Identifier::new(
            self._prepend_vendor(&Name::new(node.id)).to_owned(),
            node.range,
        );
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
            let name_parts: Vec<&str> = string_literal.value.splitn(2, ".").collect();
            let module_part = name_parts[0];
            if self.affected_names.contains(module_part) {
                string_literal.value =
                    format!("{}.{}", self.get_vendor_string(), string_literal.value)
                        .into_boxed_str();
            }
        }

        Some(string_literal)
    }

    fn visit_stmt(&mut self, stmt: ruff_python_ast::Stmt) -> Option<ruff_python_ast::Stmt> {
        match stmt {
            Stmt::Import(mut import) => {
                import.names = import
                    .names
                    .iter()
                    .map(|name| Alias {
                        range: name.range,
                        name: self._prepend_vendor_import(
                            name.name.to_owned(),
                            name.name.as_str(),
                            name.asname.is_none(),
                        ),
                        asname: name.asname.to_owned(),
                    })
                    .collect();
                Some(Stmt::Import(import))
            }
            Stmt::ImportFrom(mut import_from) => {
                if import_from.level == 0 {
                    if let Some(module_node) = &import_from.module {
                        let module_spec = module_node.as_str();
                        let new_module =
                            self._prepend_vendor_import(module_node.to_owned(), module_spec, false);
                        import_from.module = Option::from(new_module);
                    }
                }
                Some(Stmt::ImportFrom(import_from))
            }
            _ => walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, expr: Expr) -> Option<ruff_python_ast::Expr> {
        match expr {
            Expr::Name(mut name) => {
                if self.affected_names.contains(name.id.as_str()) {
                    name.id = Name::new(self._prepend_vendor(&name.id));
                }
                Some(Expr::Name(name))
            }
            Expr::Attribute(mut attribute) => {
                let mut full_name = attribute.attr.to_string();

                let mut deepest_attribute = attribute.clone();
                loop {
                    let value_expr = *deepest_attribute.value.clone();

                    match value_expr {
                        Expr::Attribute(attr) => {
                            full_name = format!("{}.{}", attr.attr, full_name);
                            deepest_attribute = attr;
                        }
                        Expr::Name(name) => {
                            full_name = format!("{}.{}", name.id, full_name);
                            break;
                        }
                        _ => {
                            self.visit_expr(value_expr);
                            break;
                        }
                    }
                }

                let name_parts: Vec<&str> = full_name.rsplitn(2, ".").collect();
                let mut module_part = name_parts[0];
                if name_parts.len() > 1 {
                    module_part = name_parts[1];
                }

                if self.affected_names.contains(module_part)
                    || self
                        .affected_names
                        .iter()
                        .find(|x| module_part.starts_with(*x))
                        .is_some()
                {
                    println!("yay!");
                    attribute.attr = Identifier::new(full_name, attribute.attr.range);
                    attribute.value = Box::new(Expr::Name(ExprName {
                        range: deepest_attribute.range,
                        id: Name::new(self.get_vendor_string()),
                        ctx: deepest_attribute.ctx,
                    }));
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
