use pyo3::{
    types::{PyAnyMethods, PyModule},
    PyResult, Python,
};

pub fn get_parent_package(package: &str) -> String {
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

pub fn is_in_std_lib(module_spec: &str) -> PyResult<bool> {
    return Python::with_gil(|py| -> PyResult<bool> {
        let stdlib_list = PyModule::import(py, "stdlib_list")?;
        let result: bool = stdlib_list
            .getattr("in_stdlib")?
            .call1((module_spec,))?
            .extract()?;
        Ok(result)
    });
}
