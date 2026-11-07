use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use pyo3::{pyclass, pymethods};
use ruff_python_ast::{Alias, CmpOp, Expr, Stmt, StmtImport, StmtImportFrom};
use thin_vec::ThinVec;

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
    cache_dir: Option<PathBuf>,
}
impl NodesRemover {
    pub(crate) fn new(
        mut references_counts: HashMap<String, usize>,
        known_modules: HashSet<String>,
        cache_dir: Option<PathBuf>,
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
            cache_dir,
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
        if let Some(blob) = fs_cache::get_counted(self.cache_dir.as_deref(), &cache_key)
            && let Some((new_source, new_body_len, removed_delta)) = parse_removal_payload(&blob)
        {
            self.statements_removed += removed_delta;
            return apply_removal(&self.source_path, &new_source, new_body_len);
        }
        self.names_provider =
            FullyQualifiedNameProvider::new(&self.module_spec, self.get_source_path());
        let parsed = entry.parsed.clone();
        let module = parsed.syntax();
        self.names_provider.seed_module_names(&module.body);
        // register top-level imports so constant folding can resolve names
        // like typing.TYPE_CHECKING or sys.platform
        for stmt in &module.body {
            self.names_provider.visit_stmt(stmt);
        }
        let statements_before = self.statements_removed;
        let new_body = self.visit_body(&module.body);

        let new_source = generate_source(&new_body, &parsed, &entry.source);
        let removed_delta = self.statements_removed - statements_before;
        fs_cache::put(
            self.cache_dir.as_deref(),
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

impl NodesRemover {
    /// Fold a constant if-test to its runtime value. Handles literals,
    /// typing.TYPE_CHECKING (always False at runtime) and sys.platform
    /// comparisons, which are folded against the platform flay runs on.
    fn fold_const_test(&self, expr: &Expr) -> Option<bool> {
        match expr {
            Expr::BooleanLiteral(lit) => Some(lit.value),
            Expr::NoneLiteral(_) => Some(false),
            Expr::NumberLiteral(_) => None,
            Expr::StringLiteral(lit) => Some(!lit.value.is_empty()),
            Expr::UnaryOp(unary) if unary.op.is_not() => {
                self.fold_const_test(&unary.operand).map(|b| !b)
            }
            Expr::Name(_) | Expr::Attribute(_) => {
                let fqns = self.names_provider.get_expr_fully_qualified_name(expr);
                if fqns.iter().any(|fqn| {
                    fqn == "typing.TYPE_CHECKING" || fqn == "typing_extensions.TYPE_CHECKING"
                }) {
                    return Some(false);
                }
                if fqns.iter().any(|fqn| fqn == "sys.platform") {
                    return Some(true); // folded by callers comparing against it
                }
                None
            }
            Expr::Compare(compare) => {
                if compare.ops.len() != 1 || compare.operands.len() != 2 {
                    return None;
                }
                let platform = host_sys_platform();
                let left_is_platform = matches!(&compare.operands[0], Expr::Name(_) | Expr::Attribute(_)
                    if self.names_provider.get_expr_fully_qualified_name(&compare.operands[0]).iter().any(|f| f == "sys.platform"));
                let right_is_platform = matches!(&compare.operands[1], Expr::Name(_) | Expr::Attribute(_)
                    if self.names_provider.get_expr_fully_qualified_name(&compare.operands[1]).iter().any(|f| f == "sys.platform"));
                let literal = if left_is_platform {
                    compare.operands[1].as_string_literal_expr()
                } else if right_is_platform {
                    compare.operands[0].as_string_literal_expr()
                } else {
                    None
                };
                let literal = literal?;
                let value = literal.value.to_str();
                let result = match compare.ops[0] {
                    CmpOp::Eq => value == platform,
                    CmpOp::NotEq => value != platform,
                    _ => return None,
                };
                Some(if left_is_platform { result } else { !result })
            }
            Expr::Call(call) => {
                // sys.platform.startswith("...") / endswith("...")
                let Expr::Attribute(attr) = &*call.func else {
                    return None;
                };
                if !self
                    .names_provider
                    .get_expr_fully_qualified_name(&attr.value)
                    .iter()
                    .any(|fqn| fqn == "sys.platform")
                {
                    return None;
                }
                let method = attr.attr.as_str();
                if method != "startswith" && method != "endswith" {
                    return None;
                }
                if call.arguments.args.len() != 1 {
                    return None;
                }
                let prefix = call.arguments.args[0].as_string_literal_expr()?;
                let platform = host_sys_platform();
                Some(if method == "startswith" {
                    platform.starts_with(prefix.value.to_str())
                } else {
                    platform.ends_with(prefix.value.to_str())
                })
            }
            _ => None,
        }
    }
}

fn host_sys_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    }
}

impl Transformer for NodesRemover {
    fn visit_body(&mut self, body: &[Stmt]) -> ThinVec<Stmt> {
        let mut new_body: ThinVec<Stmt> = ThinVec::new();
        for stmt in body {
            // fold whole constant if/elif/else chains and splice the reachable
            // branch; non-constant chains are handled by visit_stmt as before
            if let Stmt::If(if_stmt) = stmt {
                let mut tests: Vec<Option<bool>> = vec![self.fold_const_test(&if_stmt.test)];
                let mut has_else = false;
                for clause in &if_stmt.elif_else_clauses {
                    match &clause.test {
                        Some(test) => tests.push(self.fold_const_test(test)),
                        None => has_else = true,
                    }
                }
                if tests.iter().all(|t| t.is_some()) {
                    self.statements_removed += 1;
                    if tests[0] == Some(true) {
                        new_body.extend(self.visit_body(&if_stmt.body));
                        continue;
                    }
                    for clause in &if_stmt.elif_else_clauses {
                        match &clause.test {
                            Some(test) => {
                                if self.fold_const_test(test) == Some(true) {
                                    new_body.extend(self.visit_body(&clause.body));
                                    break;
                                }
                            }
                            None => {
                                if has_else {
                                    new_body.extend(self.visit_body(&clause.body));
                                }
                                break;
                            }
                        }
                    }
                    continue;
                }
            }
            if let Some(new_stmt) = self.visit_stmt(stmt.to_owned()) {
                new_body.push(new_stmt);
            }
        }
        new_body
    }

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
