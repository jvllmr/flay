use std::{collections::HashSet, path::PathBuf};

use ruff_python_ast::{Expr, Stmt};

use super::imports_provider::{ImportTrackingProviderScope, ImportsTrackingProvider};

/// Tracks names bound by non-import statements so that the fully qualified name
/// provider can tell whether a name refers to an import, a builtin or to a
/// locally defined object. Names bound by imports are intentionally not
/// tracked as local; the imports tracking provider is owned by this provider
/// because import bindings and local bindings decide together what a name
/// means.
pub struct LocalityProvider {
    imports_provider: ImportsTrackingProvider,
    local_names: HashSet<String>,
    /// names bound in the outermost module scope, either by an assignment or
    /// by a `global` declaration
    module_names: HashSet<String>,
    scope_depth: usize,
}

pub struct LocalityProviderScope {
    pub local_names: Option<HashSet<String>>,
    pub imports_scope: ImportTrackingProviderScope,
}

impl LocalityProvider {
    pub fn new(module_spec: &str, source_path: &PathBuf) -> Self {
        LocalityProvider {
            imports_provider: ImportsTrackingProvider::new(module_spec, source_path),
            local_names: HashSet::new(),
            module_names: HashSet::new(),
            scope_depth: 0,
        }
    }

    pub fn get_imports_provider(&self) -> &ImportsTrackingProvider {
        &self.imports_provider
    }

    pub fn get_parent_package(&self) -> String {
        self.imports_provider.get_parent_package()
    }

    pub fn is_local(&self, name: &str) -> bool {
        self.local_names.contains(name)
    }

    pub fn is_module_local(&self, name: &str) -> bool {
        self.module_names.contains(name)
    }

    /// Record module level bindings before visiting. Import keep decisions
    /// happen when the import is visited, so the provider must know module
    /// local names before the defining statement is reached.
    pub fn seed_module_names(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            let (names, global_names) = bound_names(stmt);
            for name in names {
                self.module_names.insert(name);
            }
            for name in global_names {
                self.module_names.insert(name);
            }
        }
    }

    /// Snapshot the current local names on scope entry. Function parameters are
    /// local for the whole function body and are therefore added here, before
    /// the body is visited.
    pub fn enter_scope(&mut self, stmt: &Stmt) -> LocalityProviderScope {
        let imports_scope = self.imports_provider.enter_scope(stmt);
        match stmt {
            Stmt::FunctionDef(func_def) => {
                let scope = LocalityProviderScope {
                    local_names: Some(self.local_names.clone()),
                    imports_scope,
                };
                self.scope_depth += 1;
                let parameters = &func_def.parameters;
                for parameter_with_default in parameters
                    .posonlyargs
                    .iter()
                    .chain(parameters.args.iter())
                    .chain(parameters.kwonlyargs.iter())
                {
                    self.local_names
                        .insert(parameter_with_default.parameter.name.to_string());
                }
                if let Some(vararg) = &parameters.vararg {
                    self.local_names.insert(vararg.name.to_string());
                }
                if let Some(kwarg) = &parameters.kwarg {
                    self.local_names.insert(kwarg.name.to_string());
                }
                scope
            }
            Stmt::ClassDef(_) => {
                self.scope_depth += 1;
                LocalityProviderScope {
                    local_names: Some(self.local_names.clone()),
                    imports_scope,
                }
            }
            _ => LocalityProviderScope {
                local_names: None,
                imports_scope,
            },
        }
    }

    pub fn exit_scope(&mut self, scope: LocalityProviderScope) {
        if let Some(local_names) = scope.local_names {
            self.local_names = local_names;
            self.scope_depth -= 1;
        }
        self.imports_provider.exit_scope(scope.imports_scope);
    }

    /// Record names bound by the statement. Names bound by dynamic imports
    /// behave like imports and must not shadow them.
    pub fn visit_stmt(&mut self, stmt: &Stmt) {
        let dynamic_import_names = self.imports_provider.visit_stmt(stmt);
        let skipped: HashSet<&str> = dynamic_import_names
            .iter()
            .map(|name| name.as_str())
            .collect();
        let (new_names, new_module_names) = bound_names(stmt);
        for name in new_names {
            if skipped.contains(name.as_str()) {
                continue;
            }
            if self.scope_depth == 0 {
                self.module_names.insert(name.clone());
            }
            self.local_names.insert(name);
        }
        for name in new_module_names {
            self.local_names.insert(name.clone());
            self.module_names.insert(name);
        }
    }
}

/// Collect the names bound by a statement and the names declared global.
fn bound_names(stmt: &Stmt) -> (Vec<String>, Vec<String>) {
    let mut new_names: Vec<String> = Vec::new();
    let mut new_module_names: Vec<String> = Vec::new();
    match stmt {
        Stmt::Assign(assign) => {
            for target in &assign.targets {
                collect_target_names(target, &mut new_names);
            }
        }
        Stmt::AnnAssign(ann_assign) => {
            collect_target_names(&ann_assign.target, &mut new_names);
        }
        Stmt::AugAssign(aug_assign) => {
            collect_target_names(&aug_assign.target, &mut new_names);
        }
        Stmt::FunctionDef(func_def) => {
            new_names.push(func_def.name.to_string());
        }
        Stmt::ClassDef(class_def) => {
            new_names.push(class_def.name.to_string());
        }
        Stmt::For(for_stmt) => {
            collect_target_names(&for_stmt.target, &mut new_names);
        }
        Stmt::With(with_stmt) => {
            for item in &with_stmt.items {
                if let Some(target) = &item.optional_vars {
                    collect_target_names(target, &mut new_names);
                }
            }
        }
        Stmt::Try(try_stmt) => {
            for ruff_python_ast::ExceptHandler::ExceptHandler(handler) in &try_stmt.handlers {
                if let Some(name) = &handler.name {
                    new_names.push(name.to_string());
                }
            }
        }
        Stmt::Match(match_stmt) => {
            for case in &match_stmt.cases {
                collect_pattern_names(&case.pattern, &mut new_names);
            }
        }
        Stmt::Global(global) => {
            for name in &global.names {
                new_names.push(name.to_string());
                // a `global` declaration binds in the module scope
                new_module_names.push(name.to_string());
            }
        }
        Stmt::Nonlocal(nonlocal) => {
            for name in &nonlocal.names {
                new_names.push(name.to_string());
            }
        }
        Stmt::TypeAlias(type_alias) => {
            collect_target_names(&type_alias.name, &mut new_names);
        }
        _ => {}
    }
    (new_names, new_module_names)
}

/// Collect plain names bound by an assignment-like target. Attribute and
/// subscript targets rebind on an existing object and do not introduce a new
/// local name. Comprehension targets are expressions and are not covered.
fn collect_target_names(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::Name(name) => names.push(name.id.to_string()),
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                collect_target_names(element, names);
            }
        }
        Expr::List(list) => {
            for element in &list.elts {
                collect_target_names(element, names);
            }
        }
        Expr::Starred(starred) => collect_target_names(&starred.value, names),
        _ => {}
    }
}

fn collect_pattern_names(pattern: &ruff_python_ast::Pattern, names: &mut Vec<String>) {
    use ruff_python_ast::Pattern;
    match pattern {
        Pattern::MatchValue(_) | Pattern::MatchSingleton(_) => {}
        Pattern::MatchSequence(sequence) => {
            for pattern in &sequence.patterns {
                collect_pattern_names(pattern, names);
            }
        }
        Pattern::MatchMapping(mapping) => {
            for pattern in &mapping.patterns {
                collect_pattern_names(pattern, names);
            }
            if let Some(rest) = &mapping.rest {
                names.push(rest.to_string());
            }
        }
        Pattern::MatchClass(class) => {
            for pattern in &class.arguments.patterns {
                collect_pattern_names(pattern, names);
            }
            for keyword in &class.arguments.keywords {
                collect_pattern_names(&keyword.pattern, names);
            }
        }
        Pattern::MatchStar(star) => {
            if let Some(name) = &star.name {
                names.push(name.to_string());
            }
        }
        Pattern::MatchAs(as_pattern) => {
            if let Some(name) = &as_pattern.name {
                names.push(name.to_string());
            }
            if let Some(pattern) = &as_pattern.pattern {
                collect_pattern_names(pattern, names);
            }
        }
        Pattern::MatchOr(or_pattern) => {
            for pattern in &or_pattern.patterns {
                collect_pattern_names(pattern, names);
            }
        }
    }
}
