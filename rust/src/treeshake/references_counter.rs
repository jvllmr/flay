use std::{collections::HashMap, path::PathBuf};

use pyo3::{pyclass, pymethods};
use rustpython_ast::{Expr, ExprCompare, Stmt, StmtImport, StmtImportFrom, Visitor};

use crate::{
    common::{ast::get_import_from_absolute_module_spec, module_spec::get_parent_package},
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
};

#[pyclass]
pub struct ReferencesCounter {
    names_provider: FullyQualifiedNameProvider,
    module_spec: String,
    #[pyo3(get, set)]
    references_counts: HashMap<String, usize>,
    #[pyo3(get, set)]
    new_reference_count: usize,
    always_bump: bool,
    source_path: PathBuf,
}

#[pymethods]
impl ReferencesCounter {
    #[new]
    fn new(module_spec: &str, references_counts: HashMap<String, usize>) -> Self {
        ReferencesCounter {
            module_spec: module_spec.to_string(),
            names_provider: FullyQualifiedNameProvider::new(module_spec),
            references_counts,
            always_bump: false,
            new_reference_count: 0,
            source_path: PathBuf::new(),
        }
    }

    fn new_module_spec(&mut self, module_spec: &str) {
        self.module_spec = module_spec.to_string();
        self.names_provider = FullyQualifiedNameProvider::new(module_spec)
    }

    fn reset(&mut self, source_path: PathBuf) {
        self.new_reference_count = 0;
        self.source_path = source_path;
    }
}

impl ReferencesCounter {
    fn increase(&mut self, fqn: &str) {
        let old_references_count = self.references_counts.get(fqn);

        match old_references_count {
            Some(count) => {
                self.references_counts.insert(fqn.to_string(), count + 1);
            }
            None => {
                self.references_counts.insert(fqn.to_string(), 1);
                self.new_reference_count += 1;
            }
        }
    }

    fn module_spec_has_references(&self) -> bool {
        for (key, count) in &self.references_counts {
            if key.starts_with(&self.module_spec) && *count > 0 {
                return true;
            }
        }
        false
    }

    fn maybe_increase_stmt(&mut self, stmt: &Stmt) {
        for fqn in self.names_provider.get_stmt_fully_qualified_name(stmt) {
            self.increase(&fqn)
        }
    }

    fn maybe_increase_expr(&mut self, expr: &Expr) {
        for fqn in self.names_provider.get_expr_fully_qualified_name(expr) {
            self.increase(&fqn)
        }
    }

    fn has_references_for_stmt(&self, stmt: &Stmt) -> bool {
        for fqn in self.names_provider.get_stmt_fully_qualified_name(stmt) {
            // TODO: ??? this looks wrong; someone with more rust xp please help
            if self.references_counts.get(&fqn).unwrap_or(&(0 as usize)) > &0 {
                return true;
            }
        }
        false
    }
}

fn is_if_name_main(expr: &Expr) -> bool {
    if let Expr::Compare(ExprCompare {
        ops: cmp_ops,
        left,
        comparators,
        ..
    }) = expr
    {
        // NOTE: make more readable?
        return cmp_ops.len() == 1
            && cmp_ops[0].is_eq()
            && comparators.len() == 1
            && ((left
                .as_name_expr()
                .is_some_and(|name| name.id.as_str() == "__name__")
                && comparators[0].as_constant_expr().is_some_and(|c| {
                    c.value
                        .as_str()
                        .is_some_and(|c_value| c_value == "__main__")
                }))
                || (comparators[0]
                    .as_name_expr()
                    .is_some_and(|name| name.id.as_str() == "__name__")
                    && left.as_constant_expr().is_some_and(|c| {
                        c.value
                            .as_str()
                            .is_some_and(|c_value| c_value == "__main__")
                    })));
    }

    return false;
}

impl Visitor for ReferencesCounter {
    fn visit_stmt(&mut self, stmt: Stmt) {
        if self.always_bump {
            self.maybe_increase_stmt(&stmt);
        }
        let mut next_always_bump = false;

        match &stmt {
            Stmt::ClassDef(class_def) => {
                if class_def.decorator_list.len() > 0 || self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    next_always_bump = true;
                }
            }
            Stmt::AnnAssign(_) | Stmt::Assign(_) => {
                if self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    next_always_bump = true;
                }
            }
            Stmt::If(if_block) => {
                if is_if_name_main(&if_block.test) {
                    self.maybe_increase_stmt(&stmt);
                    next_always_bump = true;
                }
            }
            Stmt::ImportFrom(import_from) => {
                if import_from.names.len() == 1 && import_from.names[0].name.as_str() == "*" {
                    while let Ok(module_spec) = get_import_from_absolute_module_spec(
                        &import_from,
                        &get_parent_package(&self.module_spec),
                    ) {}
                }
            }
            _ => {}
        }

        let scope = self.names_provider.enter_scope(&stmt);
        self.always_bump = next_always_bump;
        self.generic_visit_stmt(stmt);
        self.always_bump = false;
        self.names_provider.exit_scope(scope);
    }

    fn visit_expr(&mut self, expr: Expr) {
        if self.always_bump {
            self.maybe_increase_expr(&expr);
        }
        self.generic_visit_expr(expr);
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
