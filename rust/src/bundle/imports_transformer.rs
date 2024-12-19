use std::collections::HashSet;

use pyo3::pyfunction;
use rustpython_ast::{
    text_size::TextRange, Identifier, StmtImport, StmtImportFrom, Suite, Visitor,
};
use rustpython_parser::Parse;

use crate::common::module_spec::{get_top_level_package, is_in_std_lib};

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
    fn _prepend_vendor(&self, node: &Identifier) -> Identifier {
        let node_str = node.as_str();
        return Identifier::from(
            self.top_level_package.to_owned() + "." + &self.vendor_module_name + "." + node_str,
        );
    }

    fn _prepend_vendor_import<'a>(
        &mut self,
        node: Identifier,
        module_spec: &str,
        references_need_update: bool,
    ) -> Identifier {
        if module_spec.starts_with(&self.top_level_package)
            || is_in_std_lib(get_top_level_package(module_spec)).unwrap()
        {
            return node.clone();
        }
        if references_need_update {
            self.affected_names.insert(node.to_string());
        }
        return self._prepend_vendor(&node).to_owned();
    }
}
// implement Fold instead
impl Visitor for ImportsTransformer {
    fn visit_stmt_import(&mut self, mut node: StmtImport<TextRange>) {
        node.names.iter_mut().for_each(|name| {
            name.name = self._prepend_vendor_import(
                name.name.to_owned(),
                name.name.as_str(),
                name.asname.is_none(),
            )
        });
    }

    fn visit_stmt_import_from(&mut self, mut node: StmtImportFrom<TextRange>) {
        if node.module.is_some() && node.level.is_none() {
            let module_node = node.module.unwrap();
            let module_spec = module_node.as_str();
            node.module = Option::from(self._prepend_vendor_import(
                module_node.to_owned(),
                module_spec,
                false,
            ));
        }
    }
}
#[pyfunction]
pub fn transform_imports(
    source: &str,
    source_path: &str,
    top_level_package: &str,
    vendor_module_name: &str,
) {
    let mut asts = Suite::parse(source, source_path).unwrap();
    asts.iter_mut().for_each(|ast| {
        let transformer = ImportsTransformer::new(
            top_level_package.to_string(),
            vendor_module_name.to_string(),
        );
        // transformer.visit_stmt(ast);
    });
}
