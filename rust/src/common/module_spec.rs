use pyo3::{
    PyResult, Python,
    types::{PyAnyMethods, PyModule},
};

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
    let result = Python::with_gil(|py| -> PyResult<bool> {
        let stdlib_list = PyModule::import(py, "flay.common.module_spec")?;
        let result: bool = stdlib_list
            .getattr("in_stdlib")?
            .call1((module_spec,))?
            .extract()?;
        Ok(result)
    });

    match result {
        Ok(result_value) => result_value,
        Err(_) => false,
    }
}
