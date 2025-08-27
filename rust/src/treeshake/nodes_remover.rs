use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use pyo3::{pyclass, pymethods};
use ruff_python_ast::{Alias, Stmt, StmtImport, StmtImportFrom};
use ruff_python_codegen::{Generator, Stylist};
use ruff_python_parser::parse_module;

use crate::common::ast::{
    generate_source, get_import_from_absolute_module_spec,
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
    transformer::{Transformer, walk_stmt},
};

use super::references_counter::ReferencesHolder;

#[pyclass]
pub struct NodesRemover {
    references_counts: HashMap<String, usize>,
    module_spec: String,
    source_path: PathBuf,
    names_provider: FullyQualifiedNameProvider,
    #[pyo3(get)]
    statements_removed: u32,
}
#[pymethods]
impl NodesRemover {
    #[new]
    fn new(mut references_counts: HashMap<String, usize>, known_modules: HashSet<String>) -> Self {
        // known modules whose members are references should also be count as referenced
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
            statements_removed: 0,
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
        let parsed = parse_module(&file_content).unwrap();
        let module = parsed.syntax();
        let new_body = self.visit_body(&module.body);

        let stylist = Stylist::from_tokens(parsed.tokens(), &file_content);
        let mut generator: Generator = (&stylist).into();
        generator.unparse_suite(&new_body);
        let new_source = generate_source(&new_body, parsed, &file_content);
        let dir_path = self.source_path.parent().unwrap();
        if new_source.len() > 0 && new_body.len() > 0 {
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

impl NodesRemover {
    fn visit_stmt_import(&mut self, mut stmt: StmtImport) -> Option<StmtImport> {
        let mut new_names: Vec<Alias> = Vec::new();
        for name in stmt.names {
            if self.has_references_for_str(&name.name) {
                new_names.push(name);
            }
        }
        stmt.names = new_names;
        Some(stmt)
    }

    fn visit_stmt_import_from(&mut self, mut stmt: StmtImportFrom) -> Option<StmtImportFrom> {
        let mut new_names: Vec<Alias> = Vec::new();
        let mut added_names: HashSet<String> = HashSet::new();
        if let Ok(module_specs) =
            get_import_from_absolute_module_spec(&stmt, &self.get_parent_package(), true)
        {
            for module_spec in &module_specs {
                for name in &stmt.names {
                    let result_name = name.asname.as_ref().unwrap_or(&name.name);
                    if !added_names.contains(result_name.as_str())
                        && (name.name.as_str() == "*"
                            || self
                                .has_references_for_str(&format!("{}.{}", module_spec, name.name)))
                    {
                        new_names.push(name.clone());
                        added_names.insert(result_name.to_string());
                    }
                }
            }
        }
        stmt.names = new_names;
        Some(stmt)
    }
    fn fallback_stmt(&mut self, stmt: Stmt) -> Option<Stmt> {
        match stmt {
            _ => None,
        }
    }
}

impl Transformer for NodesRemover {
    fn visit_stmt(&mut self, stmt: ruff_python_ast::Stmt) -> Option<Stmt> {
        let cannot_remove_stmt = match stmt {
            Stmt::ClassDef(_)
            | Stmt::FunctionDef(_)
            | Stmt::AnnAssign(_)
            | Stmt::AugAssign(_)
            | Stmt::Assign(_)
            | Stmt::Import(_)
            | Stmt::ImportFrom(_) => false,
            _ => true,
        };
        if cannot_remove_stmt {
            return Some(stmt);
        }

        if !self.has_references_for_stmt(&stmt) {
            let fallback = self.fallback_stmt(stmt);
            if fallback.is_none() {
                self.statements_removed += 1;
            }
            return fallback;
        }
        let scope = self.names_provider.enter_scope(&stmt);
        if let Some(new_stmt) = match stmt {
            Stmt::Import(import) => self.visit_stmt_import(import).map(Stmt::Import),
            Stmt::ImportFrom(import_from) => self
                .visit_stmt_import_from(import_from)
                .map(Stmt::ImportFrom),

            _ => Some(stmt),
        } {
            if let Some(walked_stmt) = walk_stmt(self, new_stmt) {
                self.names_provider.exit_scope(scope);
                return Some(walked_stmt);
            }
        }
        self.names_provider.exit_scope(scope);
        self.statements_removed += 1;
        None
    }
}
