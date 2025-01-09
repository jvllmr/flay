mod bundle;
mod common;

use bundle::file_collector::FileCollector;
use bundle::imports_transformer::transform_imports;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_flay_rs")]
mod flay {
    #[pymodule_export]
    use super::FileCollector;

    #[pymodule_export]
    use super::transform_imports;
}
