use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use pyo3::{pyclass, pymethods};
use rustpython_ast::{Expr, ExprCompare, Stmt, StmtImport, StmtImportFrom, Suite, Visitor};
use rustpython_parser::Parse;

use crate::{
    common::{
        ast::{
            full_name::{get_full_name_for_expr, get_full_name_for_stmt},
            get_import_from_absolute_module_spec,
        },
        module_spec::get_parent_package,
    },
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
};

#[pyclass]
pub struct ReferencesCounter {
    names_provider: FullyQualifiedNameProvider,
    module_spec: String,
    #[pyo3(get, set)]
    references_counts: HashMap<String, usize>,
    #[pyo3(get, set)]
    new_references_count: usize,
    always_bump_context: bool,
    source_path: PathBuf,
    import_star_module_specs: HashSet<String>,
}

#[pymethods]
impl ReferencesCounter {
    #[new]
    fn new(references_counts: HashMap<String, usize>) -> Self {
        ReferencesCounter {
            module_spec: String::new(),
            names_provider: FullyQualifiedNameProvider::new(""),
            references_counts,
            always_bump_context: false,
            new_references_count: 0,
            source_path: PathBuf::new(),
            import_star_module_specs: HashSet::new(),
        }
    }

    fn reset_counter(&mut self) {
        self.new_references_count = 0;
    }

    fn visit_module(
        &mut self,
        module_spec: String,
        source_path: PathBuf,
    ) -> Result<(), std::io::Error> {
        self.always_bump_context = false;
        self.names_provider = FullyQualifiedNameProvider::new(&module_spec);
        self.module_spec = module_spec;
        self.source_path = source_path;
        let file_content = fs::read_to_string(&self.source_path)?;
        let stmts = Suite::parse(&file_content, &self.source_path.to_str().unwrap()).unwrap();
        for stmt in stmts {
            self.visit_stmt(stmt);
        }
        Ok(())
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
                self.new_references_count += 1;
            }
        }
    }
    fn is_global_scope(&self) -> bool {
        self.names_provider.name_context.len() == 0
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
            self.increase(&fqn);
        }

        // bump for this node because it is a global name that could be imported somewhere else via star import
        if self.import_star_module_specs.len() > 0 {
            for module_spec in self.import_star_module_specs.to_owned() {
                for full_name in get_full_name_for_stmt(stmt) {
                    self.increase(&format!("{}.{}", module_spec, full_name));
                }
            }
        }
    }

    fn maybe_increase_expr(&mut self, expr: &Expr) {
        for fqn in self.names_provider.get_expr_fully_qualified_name(expr) {
            self.increase(&fqn);
        }
        // bump for this node because it is a global name that could be imported somewhere else via star import
        if self.import_star_module_specs.len() > 0 {
            if let Some(full_name) = get_full_name_for_expr(expr) {
                for module_spec in self.import_star_module_specs.to_owned() {
                    self.increase(&format!("{}.{}", module_spec, full_name));
                }
            }
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

    fn is_in_package(&self) -> bool {
        self.source_path.ends_with("__init__.py") || self.source_path.ends_with("__main__.py")
    }

    fn get_parent_package(&self) -> String {
        if self.is_in_package() {
            return self.module_spec.to_owned();
        }
        return get_parent_package(&self.module_spec);
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
        if cmp_ops.len() == 1
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
                    })))
        {
            return true;
        }
    }

    return false;
}

impl Visitor for ReferencesCounter {
    fn visit_stmt(&mut self, stmt: Stmt) {
        let can_reset_context = !self.always_bump_context;
        if self.always_bump_context {
            self.maybe_increase_stmt(&stmt);
        }

        match &stmt {
            Stmt::Expr(expr) => {
                let expr_value = *expr.value.to_owned();

                match &expr_value {
                    Expr::Call(_) => {
                        if self.is_global_scope() && self.module_spec_has_references() {
                            self.maybe_increase_expr(&expr_value);
                            self.always_bump_context = true;
                        }
                    }
                    _ => {}
                }
            }

            Stmt::ClassDef(class_def) => {
                if class_def.decorator_list.len() > 0 || self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }

            Stmt::FunctionDef(func_def) => {
                if func_def.decorator_list.len() > 0 || self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::AsyncFunctionDef(async_func_def) => {
                if async_func_def.decorator_list.len() > 0 || self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::AnnAssign(_) | Stmt::Assign(_) => {
                if self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::If(if_block) => {
                if is_if_name_main(&if_block.test) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::ImportFrom(import_from) => {
                if import_from.names.len() == 1
                    && import_from.names[0].name.as_str() == "*"
                    && self.module_spec_has_references()
                {
                    if let Ok(module_specs) = get_import_from_absolute_module_spec(
                        &import_from,
                        &self.get_parent_package(),
                    ) {
                        self.import_star_module_specs.extend(module_specs);
                    }
                }
            }
            _ => {}
        };

        let scope = self.names_provider.enter_scope(&stmt);
        self.generic_visit_stmt(stmt);
        if can_reset_context {
            self.always_bump_context = false;
        }
        self.names_provider.exit_scope(scope);
    }

    fn visit_expr(&mut self, expr: Expr) {
        if self.always_bump_context {
            self.maybe_increase_expr(&expr);
        };
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
