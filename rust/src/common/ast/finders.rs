use ruff_python_ast::{Expr, Stmt};

use crate::common::ast::{checkers::is_dynamic_import_mut, full_name::get_full_name_for_expr};

pub fn find_dynamic_import(
    stmt: &Stmt,
    importlib_module_alias: Option<&String>,
) -> Vec<(String, String)> {
    let mut calls: Vec<(String, Expr)> = match stmt {
        Stmt::AnnAssign(ann_assign) => {
            let mut collected_calls: Vec<(String, Expr)> = Vec::new();
            if let Some(value_box) = ann_assign.value.to_owned() {
                let value = *value_box;
                if let Expr::Call(_) = value {
                    for full_name in get_full_name_for_expr(&ann_assign.target) {
                        collected_calls.push((full_name, value.clone()));
                    }
                }
            }
            collected_calls
        }
        Stmt::Assign(assign) => {
            let value = *assign.value.to_owned();
            match value {
                Expr::Call(_) => assign
                    .targets
                    .iter()
                    .flat_map(|target| {
                        get_full_name_for_expr(target)
                            .iter()
                            .map(|full_name| (full_name.to_string(), value.clone()))
                            .collect::<Vec<(String, Expr)>>()
                    })
                    .collect(),
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    };

    let mut found_import: Vec<(String, String)> = Vec::new();
    for (full_name, call_expr) in &mut calls {
        if let Some(expr) = is_dynamic_import_mut(call_expr, importlib_module_alias) {
            if let Expr::StringLiteral(string_literal) = expr {
                found_import.push((
                    full_name.to_string(),
                    string_literal.value.to_str().to_string(),
                ));
            }
        }
    }

    found_import
}
