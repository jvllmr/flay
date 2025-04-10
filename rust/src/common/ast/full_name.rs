use ruff_python_ast::{Expr, Stmt};

use super::get_import_from_absolute_module_spec;
pub fn get_full_name_for_expr(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::Name(name) => vec![name.id.to_string()],
        Expr::Attribute(attr) => get_full_name_for_expr(&attr.value)
            .iter()
            .flat_map(|value| vec![value.clone(), value.to_owned() + "." + attr.attr.as_str()])
            .collect(),
        Expr::Call(call) => get_full_name_for_expr(&call.func),
        Expr::Subscript(sub) => get_full_name_for_expr(&sub.value),
        Expr::Named(named) => get_full_name_for_expr(&named.target),
        Expr::Tuple(tuple) => tuple
            .elts
            .iter()
            .flat_map(|elts_item| get_full_name_for_expr(elts_item))
            .collect(),
        _ => Vec::new(),
    }
}

pub fn get_full_name_for_stmt(stmt: &Stmt, parent_package: &str) -> Vec<String> {
    match stmt {
        Stmt::Assign(assign) => assign
            .targets
            .iter()
            .flat_map(|target| get_full_name_for_expr(target))
            .collect(),
        Stmt::AugAssign(aug_assign) => get_full_name_for_expr(&aug_assign.target),
        Stmt::AnnAssign(aug_assign) => get_full_name_for_expr(&aug_assign.target),
        Stmt::ClassDef(def) => vec![def.name.to_string()],
        Stmt::FunctionDef(def) => vec![def.name.to_string()],

        Stmt::Expr(expr) => get_full_name_for_expr(&expr.value),
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
        Stmt::TypeAlias(type_alias) => get_full_name_for_expr(&type_alias.name),
        _ => Vec::new(),
    }
}
