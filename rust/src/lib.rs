mod bundle;
mod common;
mod rustpython;
use bundle::file_collector::FileCollector;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_flay_rs")]
mod flay {
    #[pymodule_export]
    use super::FileCollector;
}
