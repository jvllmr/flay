use rustpython_ast::{Expr, Stmt, StmtImport, StmtImportFrom};

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

    fn maybe_format_with_name_context(&self, name: &str) -> String {
        if self.name_context.len() > 0 {
            format!("{}.{}", self.name_context, name)
        } else {
            name.to_string()
        }
    }

    fn get_expr_qualified_name(&self, expr: &Expr) -> Option<String> {
        get_full_name_for_expr(expr).map(|name| match expr {
            Expr::NamedExpr(_) => self.maybe_format_with_name_context(&name),
            _ => name,
        })
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
                | Stmt::FunctionDef(_)
                | Stmt::AsyncFunctionDef(_) => {
                    self.maybe_format_with_name_context(name)
                }
                _ => name.to_string(),
            })
            .collect()
    }

    fn resolve_fully_qualified_name(&self, qualified_name: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        for (key, value) in &self.imports_provider.active_imports {
            if qualified_name.contains(key) {
                result.push(qualified_name.replace(key, &value));
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
        match self.get_expr_qualified_name(expr) {
            Some(name) => self.resolve_fully_qualified_name(&name),
            None => Vec::new(),
        }
    }

    pub fn get_stmt_fully_qualified_name(&self, stmt: &Stmt) -> Vec<String> {
        self.get_stmt_qualified_name(stmt)
            .iter()
            .flat_map(|name| self.resolve_fully_qualified_name(name))
            .collect()
    }

    pub fn visit_import_from(&mut self, import_from: &StmtImportFrom) {
        self.imports_provider.visit_import_from(import_from);
    }
    pub fn visit_import(&mut self, import: &StmtImport) {
        self.imports_provider.visit_import(import);
    }

    pub fn enter_scope(&mut self, stmt: &Stmt) -> FullyQualifiedNameProviderScope {
        let full_names = get_full_name_for_stmt(stmt, &self.parent_package);
        let old_name_context: Option<TNameContext> = match stmt {
            Stmt::ClassDef(_) | Stmt::FunctionDef(_) | Stmt::AsyncFunctionDef(_) => {
                let mut ret_value: Option<TNameContext> = None;
                if full_names.len() == 1 {
                    ret_value = Some(self.name_context.clone());
                    self.name_context = format!("{}.{}", self.name_context, full_names[0]);
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
