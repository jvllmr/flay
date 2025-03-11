mod bundle;
mod common;
mod treeshake;
use bundle::file_collector::FileCollector;
use bundle::imports_transformer::transform_imports;
use pyo3::prelude::*;
use treeshake::nodes_remover::NodesRemover;
use treeshake::references_counter::ReferencesCounter;
#[pymodule]
#[pyo3(name = "_flay_rs")]
mod flay {
    #[pymodule_export]
    use super::FileCollector;

    #[pymodule_export]
    use super::ReferencesCounter;
    #[pymodule_export]
    use super::transform_imports;

    #[pymodule_export]
    use super::NodesRemover;
}
