use std::path::PathBuf;

use pyo3::{
    PyResult, Python,
    types::{PyAnyMethods, PyModule},
};
use ruff_python_stdlib::sys::{is_builtin_module, is_known_standard_library};

use crate::constants::PYTHON_MINOR_VERSION;

pub fn get_parent_package(package: &str) -> String {
    if !package.contains(".") {
        return package.to_owned();
    }
    let parts: Vec<&str> = package.split(".").collect();
    let parent_package_parts = &parts[..parts.len() - 1];

    return parent_package_parts.join(".");
}

pub fn get_top_level_package(module_spec: &str) -> &str {
    if !module_spec.contains(".") {
        return module_spec;
    }
    return module_spec.split(".").next().unwrap();
}

pub fn is_in_std_lib(module_spec: &str) -> bool {
    let top_level = get_top_level_package(module_spec);
    return is_known_standard_library(PYTHON_MINOR_VERSION, top_level)
        || is_builtin_module(PYTHON_MINOR_VERSION, top_level)
        || top_level == "__future__";
}

pub fn get_file_for_module_spec(module_spec: &str) -> Option<(String, PathBuf)> {
    let key_result = Python::with_gil(|py| -> PyResult<Option<(String, PathBuf)>> {
        let flay_common = PyModule::import(py, "flay.common.module_spec")?;
        let module_spec_obj = flay_common
            .getattr("find_module_path")?
            .call1((module_spec,))?;

        if module_spec_obj.is_none() || module_spec_obj.getattr("origin")?.is_none() {
            return Ok(None);
        }
        let origin_attr = module_spec_obj.getattr("origin")?;
        let origin: &str = origin_attr.extract()?;
        let file_path_name: String = module_spec_obj.getattr("name")?.extract()?;
        let file_path_origin = PathBuf::from(origin);
        return Ok(Some((file_path_name, file_path_origin)));
    });
    match key_result {
        Err(py_err) => {
            println!("{:?}, {}", py_err, module_spec);
            None
        }
        Ok(key) => key,
    }
}
