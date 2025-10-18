use ruff_python_ast::{Expr, Stmt};

use crate::common::ast::full_name::get_full_name_for_expr;

pub fn is_importlib_import(stmt: &Stmt) -> Option<String> {
    let mut result: Option<String> = None;
    match stmt {
        Stmt::Import(import) => {
            for name in &import.names {
                if name.name.as_str() == "importlib" {
                    result = name
                        .asname
                        .as_ref()
                        .or(Some(&name.name))
                        .map(|x| x.to_string())
                }
            }
        }
        Stmt::ImportFrom(import_from) => {
            if import_from.level == 0
                && import_from
                    .module
                    .as_ref()
                    .is_some_and(|x| x == "importlib")
            {
                return Some("importlib".to_string());
            }
        }
        _ => {}
    };

    result
}

pub fn is_dynamic_import<'a>(
    expr: &'a Expr,
    importlib_module_alias: Option<&String>,
) -> Option<&'a Expr> {
    if let Expr::Call(call) = expr {
        let full_names = get_full_name_for_expr(&expr);

        let is_dynamic = full_names.iter().any(|full_name| {
            full_name == "__import__"
                || importlib_module_alias
                    .is_some_and(|x| *full_name == format!("{}.import_module", x))
        });

        if is_dynamic {
            if let Some(expr) = call.arguments.args.first() {
                return Some(expr);
            }
        }
    }
    None
}

pub fn is_dynamic_import_mut<'a>(
    expr: &'a mut Expr,
    importlib_module_alias: Option<&String>,
) -> Option<&'a mut Expr> {
    let expr_copy = expr.clone();
    if let Expr::Call(call) = expr {
        let full_names = get_full_name_for_expr(&expr_copy);

        let is_dynamic = full_names.iter().any(|full_name| {
            full_name == "__import__"
                || importlib_module_alias
                    .is_some_and(|x| *full_name == format!("{}.import_module", x))
        });

        if is_dynamic {
            if let Some(expr) = call.arguments.args.first_mut() {
                return Some(expr);
            }
        }
    }
    None
}
