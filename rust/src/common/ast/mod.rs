pub mod full_name;
pub mod providers;
pub mod transformer;
use pyo3::{
    exceptions::{PyImportError, PyValueError},
    PyResult,
};
use rustpython_ast::StmtImportFrom;

// does the same as libcst's resolve_name
fn resolve_name(name: &str, package: &str, level: &usize) -> PyResult<String> {
    if *level == 0 {
        return Ok(name.to_string());
    }

    let mut bits: Vec<&str> = package.rsplitn(*level, ".").collect();
    bits.reverse();

    if bits.len() < *level {
        return Err(PyImportError::new_err(
            "attempted relative import beyond top-level package",
        ));
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
    if node.module.is_none() && node.level.is_none() {
        return Err(PyValueError::new_err(
            "No absolute module spec could be found for node",
        ));
    }
    if node.module.is_some() {
        let level = match node.level {
            Some(level) => level.to_usize(),
            None => 0,
        };
        let module_node = node.module.as_ref().unwrap();

        return Ok(vec![
            resolve_name(&module_node, parent_package, &level).unwrap()
        ]);
    }

    if node.level.is_some_and(|level| level.to_usize() == 1) {
        let mut result = vec![parent_package.to_string()];
        for name in &node.names {
            let name_str = name.name.to_string();
            if name_str != "*" {
                result.push(name_str);
            }
        }
        return Ok(result);
    }

    Err(PyValueError::new_err("Don't know how to handle node"))
}
