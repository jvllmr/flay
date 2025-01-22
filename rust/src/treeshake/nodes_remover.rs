use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use pyo3::{pyclass, pymethods};

use rustpython_ast::{Stmt, Suite};
use rustpython_parser::Parse;
use rustpython_unparser::Unparser;

use crate::common::ast::{
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider, transformer::Transformer,
};

use super::references_counter::ReferencesHolder;

#[pyclass]
pub struct NodesRemover {
    references_counts: HashMap<String, usize>,
    module_spec: String,
    source_path: PathBuf,
    names_provider: FullyQualifiedNameProvider,
}
#[pymethods]
impl NodesRemover {
    #[new]
    fn new(mut references_counts: HashMap<String, usize>, known_modules: HashSet<String>) -> Self {
        let mut new_keys: Vec<String> = Vec::new();

        for known_module in &known_modules {
            for (key, value) in references_counts.iter() {
                if *value > 0 && key.starts_with(known_module) {
                    new_keys.push(known_module.to_string());
                }
            }
        }
        for new_key in new_keys {
            references_counts.insert(new_key, 1);
        }

        NodesRemover {
            references_counts,
            names_provider: FullyQualifiedNameProvider::new("", ""),
            source_path: PathBuf::new(),
            module_spec: String::new(),
        }
    }

    fn process_module(
        &mut self,
        module_spec: String,
        source_path: PathBuf,
    ) -> Result<(), std::io::Error> {
        self.module_spec = module_spec;
        self.source_path = source_path;
        self.names_provider =
            FullyQualifiedNameProvider::new(&self.module_spec, &self.get_parent_package());
        let file_content = fs::read_to_string(&self.source_path)?;
        let stmts = Suite::parse(&file_content, &self.source_path.to_str().unwrap()).unwrap();
        let new_stmts = self.visit_stmt_vec(stmts);
        let mut unparser = Unparser::new();
        for stmt in new_stmts {
            unparser.unparse_stmt(&stmt);
        }
        let new_source = unparser.source;
        let dir_path = self.source_path.parent().unwrap();
        if new_source.len() > 0 {
            fs::write(&self.source_path, new_source)?;
        } else if !self.source_path.ends_with("__init__.py") || fs::read_dir(dir_path)?.count() == 1
        {
            fs::remove_file(&self.source_path)?;
            if fs::read_dir(dir_path)?.count() == 0 {
                fs::remove_dir(dir_path)?;
            }
        }

        Ok(())
    }
}

impl ReferencesHolder for NodesRemover {
    fn get_references_counts(&self) -> &HashMap<String, usize> {
        &self.references_counts
    }

    fn get_names_provider(
        &self,
    ) -> &crate::common::ast::providers::fully_qualified_name_provider::FullyQualifiedNameProvider
    {
        &self.names_provider
    }

    fn get_source_path(&self) -> &std::path::PathBuf {
        &self.source_path
    }

    fn get_module_spec(&self) -> &String {
        &self.module_spec
    }
}

impl Transformer for NodesRemover {
    fn visit_stmt(&mut self, stmt: Stmt) -> Option<Stmt> {
        if !self.has_references_for_stmt(&stmt) {
            return None;
        }
        if let Some(new_stmt) = self.generic_visit_stmt(stmt) {
            return Some(new_stmt);
        }
        None
    }
}
