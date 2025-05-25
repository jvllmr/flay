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
