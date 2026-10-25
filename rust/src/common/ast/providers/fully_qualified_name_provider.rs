use std::path::PathBuf;

use ruff_python_ast::{Expr, Stmt};
use ruff_python_stdlib::builtins::is_python_builtin;

use crate::{
    common::ast::full_name::{get_full_name_for_expr, get_full_name_for_stmt},
    constants::PYTHON_MINOR_VERSION,
};

use super::{
    imports_provider::ImportsTrackingProvider,
    locality_provider::{LocalityProvider, LocalityProviderScope},
};

type TNameContext = String;

pub struct FullyQualifiedNameProviderScope {
    pub name_context: Option<String>,
    pub locality_scope: LocalityProviderScope,
}

pub struct FullyQualifiedNameProvider {
    pub name_context: TNameContext,
    module_spec: String,
    locality_provider: LocalityProvider,
}

impl FullyQualifiedNameProvider {
    pub fn new(module_spec: &str, source_path: &PathBuf) -> Self {
        FullyQualifiedNameProvider {
            name_context: String::new(),
            locality_provider: LocalityProvider::new(module_spec, source_path),
            module_spec: module_spec.to_string(),
        }
    }

    pub fn get_imports_provider(&self) -> &ImportsTrackingProvider {
        return self.locality_provider.get_imports_provider();
    }

    pub fn get_locality_provider(&self) -> &LocalityProvider {
        return &self.locality_provider;
    }

    pub fn seed_module_names(&mut self, stmts: &[Stmt]) {
        self.locality_provider.seed_module_names(stmts);
    }

    pub fn resolve_qualified_name(&self, name: &str) -> String {
        if self.name_context.len() > 0 {
            format!("{}.{}", self.name_context, name)
        } else {
            name.to_string()
        }
    }

    fn get_expr_qualified_name(&self, expr: &Expr) -> Vec<String> {
        get_full_name_for_expr(expr)
            .iter()
            .flat_map(|name| match expr {
                Expr::Named(_) => vec![self.resolve_qualified_name(&name)],
                Expr::Name(_) | Expr::Attribute(_) => {
                    vec![self.resolve_qualified_name(&name), name.to_owned()]
                }
                _ => vec![name.to_owned()],
            })
            .collect()
    }

    fn get_stmt_qualified_name(&self, stmt: &Stmt) -> Vec<String> {
        get_full_name_for_stmt(stmt, &self.locality_provider.get_parent_package())
            .iter()
            .map(|name|
                // TODO: match outside of map
                match stmt {
                Stmt::Assign(_)
                | Stmt::AnnAssign(_)
                | Stmt::AugAssign(_)
                | Stmt::ClassDef(_)
                | Stmt::FunctionDef(_) => {
                    self.resolve_qualified_name(name)
                }
                _ => name.to_string(),
            })
            .collect()
    }

    /// Remove the name context prefix added by `resolve_qualified_name` to get
    /// the plain name a statement or expression used.
    fn strip_name_context<'a>(&self, qualified_name: &'a str) -> &'a str {
        if self.name_context.len() > 0 {
            if let Some(rest) = qualified_name.strip_prefix(&self.name_context) {
                if let Some(rest) = rest.strip_prefix('.') {
                    return rest;
                }
            }
        }
        return qualified_name;
    }

    pub fn resolve_fully_qualified_name(&self, qualified_name: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        // a locally bound name shadows imports, star imports and builtins; it
        // can only refer to the object defined in this module
        let raw_name = self.strip_name_context(qualified_name);
        let root = raw_name.split('.').next().unwrap_or(raw_name);
        let is_local = self.locality_provider.is_local(root);

        if !is_local {
            for (key, value) in &self.locality_provider.get_imports_provider().active_imports {
                if (qualified_name.len() == key.len() && qualified_name == key)
                    || qualified_name.starts_with(&format!("{}.", key))
                {
                    result.push(qualified_name.replacen(key, &value, 1));
                }
            }
        }

        if result.len() == 0 {
            // the raw name of a name that is only bound inside a scope never
            // refers to a module level object
            let in_scope = qualified_name != raw_name;
            if !is_local || in_scope || self.locality_provider.is_module_local(root) {
                result.push(format!("{}.{}", self.module_spec, qualified_name));
                if !is_local && is_python_builtin(qualified_name, PYTHON_MINOR_VERSION, false) {
                    result.push(format!("__builtin__.{}", qualified_name));
                }
            }
        }

        if !is_local {
            for star_import in &self
                .locality_provider
                .get_imports_provider()
                .active_star_imports
            {
                result.push(format!("{}.{}", star_import, qualified_name));
            }
        }

        return result;
    }

    pub fn get_expr_fully_qualified_name(&self, expr: &Expr) -> Vec<String> {
        match expr {
            _ => self
                .get_expr_qualified_name(expr)
                .iter()
                .flat_map(|name| self.resolve_fully_qualified_name(name))
                .collect(),
        }
    }

    pub fn get_stmt_fully_qualified_name(&self, stmt: &Stmt) -> Vec<String> {
        match stmt {
            Stmt::Import(_) | Stmt::ImportFrom(_) => {
                get_full_name_for_stmt(stmt, &self.locality_provider.get_parent_package())
            }
            _ => self
                .get_stmt_qualified_name(stmt)
                .iter()
                .flat_map(|name| self.resolve_fully_qualified_name(name))
                .collect(),
        }
    }

    pub fn visit_stmt(&mut self, stmt: &Stmt) {
        self.locality_provider.visit_stmt(stmt);
    }

    pub fn enter_scope(&mut self, stmt: &Stmt) -> FullyQualifiedNameProviderScope {
        let old_name_context: Option<TNameContext> = match stmt {
            Stmt::ClassDef(_) | Stmt::FunctionDef(_) => {
                let full_names =
                    get_full_name_for_stmt(stmt, &self.locality_provider.get_parent_package());
                let mut ret_value: Option<TNameContext> = None;
                if full_names.len() == 1 {
                    ret_value = Some(self.name_context.clone());
                    if self.name_context.len() > 0 {
                        self.name_context = format!("{}.{}", self.name_context, full_names[0]);
                    } else {
                        self.name_context = full_names[0].clone();
                    }
                }

                ret_value
            }
            _ => None,
        };

        FullyQualifiedNameProviderScope {
            name_context: old_name_context,
            locality_scope: self.locality_provider.enter_scope(stmt),
        }
    }

    pub fn exit_scope(&mut self, scope: FullyQualifiedNameProviderScope) {
        if let Some(name_context) = scope.name_context {
            self.name_context = name_context;
        }
        self.locality_provider.exit_scope(scope.locality_scope);
    }
}
