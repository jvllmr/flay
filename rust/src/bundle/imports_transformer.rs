use crate::common::module_spec::{get_top_level_package, is_in_std_lib};
use pyo3::pyfunction;
use rustpython_ast::{
    text_size::TextRange, Alias, Expr, ExprAttribute, ExprName, Identifier, StmtImport,
    StmtImportFrom, Suite,
};
use rustpython_parser::Parse;
use rustpython_unparser::{transformer::Transformer, Unparser};
use std::collections::HashSet;

struct ImportsTransformer {
    top_level_package: String,
    vendor_module_name: String,
    affected_names: HashSet<String>,
}

impl ImportsTransformer {
    fn new(top_level_package: String, vendor_module_name: String) -> Self {
        ImportsTransformer {
            top_level_package,
            vendor_module_name,
            affected_names: HashSet::new(),
        }
    }

    fn get_vendor_string(&self) -> String {
        return self.top_level_package.to_owned() + "." + &self.vendor_module_name;
    }

    fn _prepend_vendor(&self, node: &Identifier) -> Identifier {
        let node_str = node.as_str();
        return Identifier::from(self.get_vendor_string() + "." + node_str);
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
        return self._prepend_vendor(&node).to_owned();
    }
}

impl Transformer for ImportsTransformer {
    fn generic_visit_stmt_import(&mut self, stmt: StmtImport) -> Option<StmtImport> {
        Some(StmtImport {
            names: stmt
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
                .collect(),
            range: stmt.range,
        })
    }

    fn visit_stmt_import_from(
        &mut self,
        node: StmtImportFrom<TextRange>,
    ) -> Option<StmtImportFrom> {
        if node.module.is_some() && node.level.is_none_or(|v| v.to_usize() == 0) {
            let module_node = node.module.unwrap();
            let module_spec = module_node.as_str();
            let new_module = Option::from(self._prepend_vendor_import(
                module_node.to_owned(),
                module_spec,
                false,
            ));
            return Some(StmtImportFrom {
                level: node.level,
                names: node.names,
                range: node.range,
                module: new_module,
            });
        }
        Some(node)
    }

    fn visit_expr_attribute(&mut self, node: ExprAttribute<TextRange>) -> Option<ExprAttribute> {
        let mut full_name = node.attr.to_string();

        let mut deepest_attribute = node.clone();
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
        if self.affected_names.contains(module_part) {
            let new_attribute = ExprAttribute {
                attr: Identifier::from(full_name),
                range: node.range,
                ctx: node.ctx,
                value: Box::new(Expr::Name(ExprName {
                    range: deepest_attribute.range,
                    id: Identifier::from(self.get_vendor_string()),
                    ctx: deepest_attribute.ctx,
                })),
            };
            return Some(new_attribute);
        }

        Some(node)
    }

    fn visit_expr_name(&mut self, node: ExprName<TextRange>) -> Option<ExprName> {
        if self.affected_names.contains(node.id.as_str()) {
            return Some(ExprName {
                id: self._prepend_vendor(&node.id),
                ctx: node.ctx,
                range: node.range,
            });
        }
        Some(node)
    }
}

#[pyfunction]
pub fn transform_imports(
    source: &str,
    source_path: &str,
    top_level_package: &str,
    vendor_module_name: &str,
) -> String {
    let stmts = Suite::parse(source, source_path).unwrap();
    let mut unparser = Unparser::new();
    let mut transformer = ImportsTransformer::new(
        top_level_package.to_string(),
        vendor_module_name.to_string(),
    );
    stmts.iter().for_each(|stmt| {
        let new_stmt = transformer.visit_stmt(stmt.to_owned()).unwrap();
        unparser.unparse_stmt(&new_stmt);
    });
    let new_source = unparser.source;
    return new_source;
}
