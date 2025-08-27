use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::pyclass;
use ruff_python_ast::Stmt;
use ruff_python_ast::visitor::Visitor;
use ruff_python_ast::visitor::walk_stmt;

use crate::common::ast::{get_import_from_absolute_module_spec, parse_python_source};
use crate::common::module_spec::{get_file_for_module_spec, get_parent_package, is_in_std_lib};

#[pyclass]
pub struct FileCollector {
    #[pyo3(get, set)]
    package: String,
    #[pyo3(get, set)]
    collected_files: HashMap<(String, PathBuf), Option<String>>,
}
#[pymethods]
impl FileCollector {
    #[new]
    fn new(package: String) -> Self {
        FileCollector {
            package,
            collected_files: HashMap::new(),
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
    fn visit_stmt(&mut self, stmt: &'_ ruff_python_ast::Stmt) {
        match stmt {
            Stmt::Import(import) => {
                for name in &import.names {
                    self._process_module(&name.name);
                }
            }
            Stmt::ImportFrom(import_from) => {
                for absolute_module_spec in
                    get_import_from_absolute_module_spec(&import_from, &self.package, true).unwrap()
                {
                    self._process_module(&absolute_module_spec);
                    // imported name could be a module

                    for name in &import_from.names {
                        if name.name.as_str() != "*" {
                            let potential_module_spec =
                                format!("{}.{}", absolute_module_spec, name.name);

                            self._process_module(&potential_module_spec);
                        }
                    }
                }
            }
            _ => {
                walk_stmt(self, stmt);
            }
        }
    }
}
