mod bundle;
mod common;
mod providers;
mod treeshake;
use bundle::file_collector::FileCollector;
use bundle::imports_transformer::transform_imports;
use pyo3::prelude::*;
use treeshake::references_counter::ReferencesCounter;

#[pymodule]
#[pyo3(name = "_flay_rs")]
mod flay {
    #[pymodule_export]
    use super::FileCollector;

    #[pymodule_export]
    use super::transform_imports;
    #[pymodule_export]
    use super::ReferencesCounter;
}
