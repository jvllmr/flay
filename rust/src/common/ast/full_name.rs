use rustpython_ast::{Expr, Stmt};

use super::get_import_from_absolute_module_spec;
pub fn get_full_name_for_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(name.id.to_string()),
        Expr::Attribute(attr) => {
            get_full_name_for_expr(&attr.value).map(|value| value + "." + attr.attr.as_str())
        }
        Expr::Call(call) => get_full_name_for_expr(&call.func),
        Expr::Subscript(sub) => get_full_name_for_expr(&sub.value),
        Expr::NamedExpr(named) => get_full_name_for_expr(&named.target),
        _ => None,
    }
}

pub fn get_full_name_for_stmt(stmt: &Stmt, parent_package: &str) -> Vec<String> {
    match stmt {
        Stmt::Assign(assign) => assign
            .targets
            .iter()
            .filter_map(|target| get_full_name_for_expr(target))
            .collect(),
        Stmt::AugAssign(aug_assign) => match get_full_name_for_expr(&aug_assign.target) {
            Some(name) => vec![name],
            None => Vec::new(),
        },
        Stmt::AnnAssign(aug_assign) => match get_full_name_for_expr(&aug_assign.target) {
            Some(name) => vec![name],
            None => Vec::new(),
        },
        Stmt::ClassDef(def) => vec![def.name.to_string()],
        Stmt::FunctionDef(def) => vec![def.name.to_string()],
        Stmt::AsyncFunctionDef(def) => vec![def.name.to_string()],
        Stmt::Expr(expr) => match get_full_name_for_expr(&expr.value) {
            Some(name) => vec![name],
            None => Vec::new(),
        },
        Stmt::Nonlocal(nonlocal) => nonlocal.names.iter().map(|id| id.to_string()).collect(),
        Stmt::Global(global) => global.names.iter().map(|id| id.to_string()).collect(),
        Stmt::Import(import) => import
            .names
            .iter()
            .map(|alias| alias.name.to_string())
            .collect(),
        Stmt::ImportFrom(import_from) => {
            let mut names: Vec<String> = Vec::new();
            if let Ok(module_specs) =
                get_import_from_absolute_module_spec(import_from, parent_package)
            {
                for module_spec in module_specs {
                    for alias in &import_from.names {
                        names.push(format!("{}.{}", module_spec, alias.name));
                    }
                }
            }
            return names;
        }
        Stmt::TypeAlias(type_alias) => match get_full_name_for_expr(&type_alias.name) {
            Some(name) => vec![name],
            None => Vec::new(),
        },
        _ => Vec::new(),
    }
}
