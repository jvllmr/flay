use pyo3::pyfunction;
use rustpython_ast::{
    text_size::TextRange, Alias, Fold, Identifier, Stmt, StmtImport, StmtImportFrom, Suite,
};
use rustpython_parser::{Parse, ParseError};
use std::collections::HashSet;

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

impl Fold<TextRange> for ImportsTransformer {
    fn fold_stmt_import(
        &mut self,
        node: StmtImport<TextRange>,
    ) -> Result<StmtImport<TextRange>, ParseError> {
        Ok(StmtImport {
            names: node
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
            range: node.range,
        })
    }

    fn fold_stmt_import_from(
        &mut self,
        node: StmtImportFrom<TextRange>,
    ) -> Result<StmtImportFrom<TextRange>, ParseError> {
        if node.module.is_some() && node.level.is_none() {
            let module_node = node.module.unwrap();
            let module_spec = module_node.as_str();
            let new_module = Option::from(self._prepend_vendor_import(
                module_node.to_owned(),
                module_spec,
                false,
            ));
            return Ok(StmtImportFrom {
                level: node.level,
                names: node.names,
                range: node.range,
                module: new_module,
            });
        }
        Ok(node)
    }

    type TargetU = TextRange;

    type Error = ParseError;

    type UserContext = bool;

    fn will_map_user(&mut self, _user: &TextRange) -> Self::UserContext {
        return false;
    }

    fn map_user(
        &mut self,
        user: TextRange,
        _context: Self::UserContext,
    ) -> Result<Self::TargetU, Self::Error> {
        Ok(user)
    }
}
#[pyfunction]
pub fn transform_imports(
    source: &str,
    source_path: &str,
    top_level_package: &str,
    vendor_module_name: &str,
) {
    let asts = Suite::parse(source, source_path).unwrap();
    let mut new_asts: Vec<Stmt> = Vec::new();
    // let unparser = Unparser { f: ??? };
    asts.iter().for_each(|ast| {
        let mut transformer = ImportsTransformer::new(
            top_level_package.to_string(),
            vendor_module_name.to_string(),
        );
        let new_ast = transformer.fold_stmt(ast.to_owned());
        new_asts.push(new_ast.unwrap());
    });
}
