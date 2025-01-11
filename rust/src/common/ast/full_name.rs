use rustpython_ast::{Expr, Stmt};
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

pub fn get_full_name_for_stmt(stmt: &Stmt) -> Vec<String> {
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
        _ => Vec::new(),
    }
}
