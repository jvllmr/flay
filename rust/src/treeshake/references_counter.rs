use pyo3::{pyclass, pymethods};
use rustpython_ast::{Stmt, StmtImport, StmtImportFrom, Visitor};

use crate::providers::fully_qualified_name_provider::FullyQualifiedNameProvider;

#[pyclass]
pub struct ReferencesCounter {
    names_provider: FullyQualifiedNameProvider,
    module_spec: String,
}

#[pymethods]
impl ReferencesCounter {
    #[new]
    fn new(module_spec: &str) -> Self {
        ReferencesCounter {
            module_spec: module_spec.to_string(),
            names_provider: FullyQualifiedNameProvider::new(module_spec),
        }
    }

    fn new_module_spec(&mut self, module_spec: &str) {
        self.module_spec = module_spec.to_string();
        self.names_provider = FullyQualifiedNameProvider::new(module_spec)
    }
}

impl Visitor for ReferencesCounter {
    fn visit_stmt(&mut self, stmt: Stmt) {
        let scope = self.names_provider.enter_scope(&stmt);
        self.generic_visit_stmt(stmt);
        self.names_provider.exit_scope(scope);
    }
    fn visit_stmt_import(&mut self, node: StmtImport) {
        self.names_provider.visit_import(&node);
        self.generic_visit_stmt_import(node);
    }

    fn visit_stmt_import_from(&mut self, node: StmtImportFrom) {
        self.names_provider.visit_import_from(&node);
        self.generic_visit_stmt_import_from(node);
    }
}
