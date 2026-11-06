use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use pyo3::{pyclass, pymethods};
use ruff_python_ast::{Alias, Stmt, StmtImport, StmtImportFrom};

use crate::common::ast::{
    generate_source, get_import_from_absolute_module_spec,
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
    transformer::{Transformer, walk_stmt},
};
use crate::common::cache::load_module;
use crate::common::fs_cache;

use super::references_counter::ReferencesHolder;

#[pyclass]
pub struct NodesRemover {
    references_counts: HashMap<String, usize>,
    module_spec: String,
    source_path: PathBuf,
    names_provider: FullyQualifiedNameProvider,
    #[pyo3(get)]
    statements_removed: u32,
    /// hash of the (already known-module-adjusted) reference counts and the
    /// known modules; part of every per-file persistent cache key
    cache_key_prefix: String,
}
impl NodesRemover {
    pub(crate) fn new(
        mut references_counts: HashMap<String, usize>,
        known_modules: HashSet<String>,
    ) -> Self {
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

        // computed after the known-module adjustment, since the per-file
        // removal results depend on the adjusted counts
        let counts_hash = fs_cache::hash_parts(&[&fs_cache::serialize_str_map(&references_counts)]);
        let mut known: Vec<&str> = known_modules.iter().map(|s| s.as_str()).collect();
        known.sort_unstable();
        let known_hash = fs_cache::hash_parts(&[known.join("\n").as_bytes()]);

        NodesRemover {
            references_counts,
            names_provider: FullyQualifiedNameProvider::new("", &PathBuf::from("")),
            source_path: PathBuf::new(),
            module_spec: String::new(),
            statements_removed: 0,
            cache_key_prefix: format!("{counts_hash}{known_hash}"),
        }
    }
}

#[pymethods]
impl NodesRemover {
    fn process_module(
        &mut self,
        module_spec: String,
        source_path: PathBuf,
    ) -> Result<(), std::io::Error> {
        self.module_spec = module_spec.clone();
        self.source_path = source_path.clone();
        let entry = load_module(&self.source_path)?;
        let file_name = self
            .source_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        // the removal result of a module is a pure function of its content,
        // its module spec / file name and the global reference counts; a leaf
        // change busts it only through the counts hash in cache_key_prefix
        let cache_key = fs_cache::hash_parts(&[
            entry.hash.as_bytes(),
            module_spec.as_bytes(),
            file_name.as_bytes(),
            self.cache_key_prefix.as_bytes(),
        ]);
        if let Some(blob) = fs_cache::get_counted(&cache_key) {
            if let Some((new_source, new_body_len, removed_delta)) = parse_removal_payload(&blob) {
                self.statements_removed += removed_delta;
                return apply_removal(&self.source_path, &new_source, new_body_len);
            }
        }
        self.names_provider =
            FullyQualifiedNameProvider::new(&self.module_spec, self.get_source_path());
        let parsed = entry.parsed.clone();
        let module = parsed.syntax();
        self.names_provider.seed_module_names(&module.body);
        let statements_before = self.statements_removed;
        let new_body = self.visit_body(&module.body);

        let new_source = generate_source(&new_body, &parsed, &entry.source);
        let removed_delta = self.statements_removed - statements_before;
        fs_cache::put(
            &cache_key,
            &removal_payload(&new_source, new_body.len(), removed_delta),
        );
        apply_removal(&self.source_path, &new_source, new_body.len())
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
        if let Ok(module_specs) = get_import_from_absolute_module_spec(
            &stmt,
            &self
                .names_provider
                .get_imports_provider()
                .get_parent_package(),
            true,
        ) {
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
            | Stmt::Assign(_) => false,
            Stmt::Import(ref import) => import
                .names
                .iter()
                .any(|alias| alias.name.as_str() == "__future__"),
            Stmt::ImportFrom(ref import_from) => {
                import_from.level == 0
                    && import_from
                        .module
                        .as_ref()
                        .is_some_and(|m| m.as_str() == "__future__")
            }
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

/// payload layout: "<new_body_len> <removed_delta>\n<new_source>"
fn removal_payload(new_source: &str, new_body_len: usize, removed_delta: u32) -> Vec<u8> {
    let mut out = format!("{new_body_len} {removed_delta}\n").into_bytes();
    out.extend(new_source.as_bytes());
    out
}

fn parse_removal_payload(blob: &[u8]) -> Option<(String, usize, u32)> {
    let newline = blob.iter().position(|byte| *byte == b'\n')?;
    let (header, source) = blob.split_at(newline);
    let source = &source[1..];
    let mut header_parts = std::str::from_utf8(header).ok()?.split(' ');
    let new_body_len = header_parts.next()?.parse().ok()?;
    let removed_delta = header_parts.next()?.parse().ok()?;
    Some((
        String::from_utf8_lossy(source).into_owned(),
        new_body_len,
        removed_delta,
    ))
}

/// Apply a transform result to the file on disk. The runtime directory
/// conditions are re-evaluated here, so a cache hit behaves exactly like a
/// cache miss.
fn apply_removal(
    source_path: &Path,
    new_source: &str,
    new_body_len: usize,
) -> Result<(), std::io::Error> {
    let dir_path = source_path.parent().unwrap();
    if !new_source.is_empty() && new_body_len > 0 {
        fs::write(source_path, new_source)?;
    } else if !source_path.ends_with("__init__.py") || fs::read_dir(dir_path)?.count() == 1 {
        fs::remove_file(source_path)?;
        if fs::read_dir(dir_path)?.count() == 0 {
            fs::remove_dir(dir_path)?;
        }
    }
    Ok(())
}
