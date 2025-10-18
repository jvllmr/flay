use ruff_python_ast::{Expr, Stmt};

use crate::common::{
    ast::full_name::{get_full_name_for_expr, get_full_name_for_stmt},
    module_spec::get_parent_package,
};

use super::imports_provider::{ImportTrackingProviderScope, ImportsTrackingProvider};

type TNameContext = String;

pub struct FullyQualifiedNameProviderScope {
    pub name_context: Option<String>,
    pub import_scope: ImportTrackingProviderScope,
}

pub struct FullyQualifiedNameProvider {
    pub name_context: TNameContext,
    module_spec: String,
    parent_package: String,
    imports_provider: ImportsTrackingProvider,
}

impl FullyQualifiedNameProvider {
    pub fn new(module_spec: &str, parent_package: &str) -> Self {
        FullyQualifiedNameProvider {
            name_context: String::new(),
            imports_provider: ImportsTrackingProvider::new(&get_parent_package(module_spec)),
            module_spec: module_spec.to_string(),
            parent_package: parent_package.to_string(),
        }
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
        get_full_name_for_stmt(stmt, &self.parent_package)
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

    pub fn resolve_fully_qualified_name(&self, qualified_name: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        for (key, value) in &self.imports_provider.active_imports {
            if (qualified_name.len() == key.len() && qualified_name == key)
                || qualified_name.starts_with(&format!("{}.", key))
            {
                result.push(qualified_name.replacen(key, &value, 1));
            }
        }

        if result.len() == 0 {
            result.push(format!("{}.{}", self.module_spec, qualified_name));
        }

        for star_import in &self.imports_provider.active_star_imports {
            result.push(format!("{}.{}", star_import, qualified_name));
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
                get_full_name_for_stmt(stmt, &self.parent_package)
            }
            _ => self
                .get_stmt_qualified_name(stmt)
                .iter()
                .flat_map(|name| self.resolve_fully_qualified_name(name))
                .collect(),
        }
    }

    pub fn visit_stmt(&mut self, stmt: &Stmt) {
        self.imports_provider.visit_stmt(stmt);
    }

    pub fn enter_scope(&mut self, stmt: &Stmt) -> FullyQualifiedNameProviderScope {
        let old_name_context: Option<TNameContext> = match stmt {
            Stmt::ClassDef(_) | Stmt::FunctionDef(_) => {
                let full_names = get_full_name_for_stmt(stmt, &self.parent_package);
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
            import_scope: self.imports_provider.enter_scope(stmt),
        }
    }

    pub fn exit_scope(&mut self, scope: FullyQualifiedNameProviderScope) {
        if let Some(name_context) = scope.name_context {
            self.name_context = name_context;
        }
        self.imports_provider.exit_scope(scope.import_scope);
    }
}
