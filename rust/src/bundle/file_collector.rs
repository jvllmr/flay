use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::pyclass;
use ruff_python_ast::Expr;
use ruff_python_ast::Stmt;
use ruff_python_ast::visitor::Visitor;
use ruff_python_ast::visitor::walk_expr;
use ruff_python_ast::visitor::walk_stmt;

use crate::common::ast::checkers::is_dynamic_import;
use crate::common::ast::checkers::is_importlib_import;
use crate::common::ast::{get_import_from_absolute_module_spec, parse_python_source};
use crate::common::module_spec::remove_last_segment;
use crate::common::module_spec::{get_file_for_module_spec, get_parent_package, is_in_std_lib};

#[pyclass]
pub struct FileCollector {
    #[pyo3(get, set)]
    package: String,
    #[pyo3(get, set)]
    collected_files: HashMap<(String, PathBuf), Option<String>>,
    importlib_package_alias: Option<String>,
    import_aliases: HashMap<String, String>,
    module_aliases: HashMap<String, HashSet<String>>,
}
#[pymethods]
impl FileCollector {
    #[new]
    fn new(package: String, import_aliases: HashMap<String, String>) -> Self {
        let mut module_aliases: HashMap<String, HashSet<String>> = HashMap::new();
        for (search, replacement) in &import_aliases {
            let (search_module, replacement_module) = (
                remove_last_segment(search),
                remove_last_segment(replacement),
            );
            match module_aliases.get_mut(search_module) {
                Some(replacements) => {
                    replacements.insert(replacement_module.to_owned());
                }
                None => {
                    module_aliases.insert(
                        search_module.to_owned(),
                        HashSet::from([replacement_module.to_owned()]),
                    );
                }
            };
        }

        FileCollector {
            package,
            collected_files: HashMap::new(),
            importlib_package_alias: None,
            import_aliases: import_aliases,
            module_aliases: module_aliases,
        }
    }

    fn _process_module(&mut self, module_spec: &str) {
        if is_in_std_lib(module_spec) {
            return;
        }

        let key_option = get_file_for_module_spec(module_spec);
        match key_option {
            None => {}
            Some(key) => {
                if self.collected_files.contains_key(&key) {
                    return;
                }

                let (module_name, file_origin) = key.to_owned();

                if file_origin
                    .extension()
                    .is_some_and(|extension| extension == "py")
                {
                    if let Ok(file_content) = read_to_string(&file_origin) {
                        self.collected_files.insert(key, Some(file_content.clone()));

                        let mut next_parent_package = get_parent_package(&module_name).to_string();
                        if file_origin.file_name().is_some_and(|file_name| {
                            file_name == "__init__.py" || file_name == "__main__.py"
                        }) {
                            next_parent_package = module_name
                        }
                        let mut sub_collector = FileCollector {
                            package: next_parent_package,
                            collected_files: self.collected_files.to_owned(),
                            importlib_package_alias: None,
                            import_aliases: self.import_aliases.to_owned(),
                            module_aliases: self.module_aliases.to_owned(),
                        };
                        let module = parse_python_source(&file_content).unwrap().expect_module();
                        for stmt in &module.body {
                            sub_collector.visit_stmt(stmt);
                        }
                        self.collected_files.extend(sub_collector.collected_files);
                    }
                } else {
                    self.collected_files.insert(key, None);
                }
            }
        };
    }
}

impl Visitor<'_> for FileCollector {
    fn visit_expr(&mut self, expr: &'_ Expr) {
        if let Some(dynamic_import_expr) =
            is_dynamic_import(expr, self.importlib_package_alias.as_ref())
        {
            match dynamic_import_expr {
                Expr::StringLiteral(literal) => {
                    self._process_module(literal.value.to_str());
                }
                _ => {}
            }
        }
        walk_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'_ ruff_python_ast::Stmt) {
        if let Some(importlib_package_alias) = is_importlib_import(stmt) {
            self.importlib_package_alias = Some(importlib_package_alias)
        }
        match stmt {
            Stmt::Import(import) => {
                let mut modules: HashSet<String> = HashSet::new();

                for name in &import.names {
                    modules.insert(name.name.to_string());
                    self._process_module(&name.name);
                    if let Some(aliases) = self.module_aliases.get(name.name.as_str()) {
                        modules.extend(aliases.iter().cloned());
                    }
                }

                for module in modules {
                    self._process_module(&module);
                }
            }
            Stmt::ImportFrom(import_from) => {
                for absolute_module_spec in
                    get_import_from_absolute_module_spec(&import_from, &self.package, true).unwrap()
                {
                    // imported name could be a module
                    let mut potential_modules: HashSet<String> =
                        HashSet::from([absolute_module_spec.clone()]);

                    if let Some(alias) = self.import_aliases.get(&absolute_module_spec) {
                        potential_modules.insert(alias.to_owned());
                    }

                    for name in &import_from.names {
                        if name.name.as_str() != "*" {
                            let potential_module_spec =
                                format!("{}.{}", absolute_module_spec, name.name);
                            potential_modules.insert(potential_module_spec.clone());
                            if let Some(alias) = self.import_aliases.get(&potential_module_spec) {
                                potential_modules.insert(alias.to_owned());
                            }
                            if let Some(aliases) = self.module_aliases.get(&potential_module_spec) {
                                potential_modules.extend(aliases.iter().cloned());
                            }
                        }
                    }

                    for potential_module in potential_modules {
                        self._process_module(&potential_module);
                    }
                }
            }
            _ => {
                walk_stmt(self, stmt);
            }
        }
    }
}
