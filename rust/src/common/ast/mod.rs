pub mod full_name;
pub mod providers;
pub mod visitor_patch;
use pyo3::{
    exceptions::{PyImportError, PyValueError},
    PyResult,
};
use rustpython_ast::StmtImportFrom;

use crate::common::module_spec::get_parent_package;

// does the same as libcst's resolve_name
fn resolve_name(name: &str, package: &str, level: &usize) -> PyResult<String> {
    if *level == 0 {
        return Ok(name.to_string());
    }

    let mut bits: Vec<&str> = package.rsplitn(*level, ".").collect();
    bits.reverse();

    if bits.len() < *level {
        return Err(PyImportError::new_err(format!(
            "attempted relative import beyond top-level package {} {}",
            &package, &level
        )));
    }
    let base = bits[0].to_string();
    if name.len() > 0 {
        return Ok(base + "." + name);
    }

    return Ok(base);
}

pub fn get_import_from_absolute_module_spec(
    node: &StmtImportFrom,
    parent_package: &str,
) -> PyResult<Vec<String>> {
    let mut fixed_parent_package = parent_package.to_string();
    if fixed_parent_package.ends_with(".__main__") {
        fixed_parent_package = fixed_parent_package.replace(".__main__", "");
    }
    if node.module.is_none() && node.level.is_none() {
        return Err(PyValueError::new_err(
            "No absolute module spec could be found for node",
        ));
    }
    if let Some(module) = &node.module {
        let level = match node.level {
            Some(level) => level.to_usize(),
            None => 0,
        };

        return Ok(vec![resolve_name(module, &fixed_parent_package, &level)?]);
    }

    if let Some(int_level) = &node.level {
        let level = int_level.to_usize();
        if level > 0 {
            let mut target_package = fixed_parent_package.clone();
            for _ in 1..level {
                target_package = get_parent_package(&target_package);
            }
            let mut result = vec![target_package.clone()];
            for name in &node.names {
                let name_str = name.name.to_string();
                if name_str != "*" {
                    result.push(format!("{}.{}", target_package, name_str));
                }
            }
            return Ok(result);
        }
    }

    Err(PyValueError::new_err("Don't know how to handle node"))
}
