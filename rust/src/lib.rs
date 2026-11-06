mod bundle;
mod common;
mod constants;
mod treeshake;
use bundle::annotations_remover::AnnotationsRemover;
use bundle::file_collector::FileCollector;

use pyo3::prelude::*;
use treeshake::nodes_remover::NodesRemover;
use treeshake::references_counter::ReferencesCounter;

#[pyfunction]
fn clear_module_cache() {
    common::cache::clear_module_cache();
}

#[pyfunction]
fn module_cache_len() -> usize {
    common::cache::module_cache_len()
}

/// (hits, misses) of the persistent, content-addressed fs cache
#[pyfunction]
fn fs_cache_stats() -> (u64, u64) {
    common::fs_cache::stats()
}

/// Route rust log records to python logging; the record target is used as the
/// python logger name, so e.g. target "flay.treeshake.package" lands on the
/// same logger as `logging.getLogger("flay.treeshake.package")` in python.
/// Only the logger objects are cached, not their levels, so runtime level
/// changes on the python side are picked up dynamically.
#[pymodule]
#[pyo3(name = "_flay_rs")]
fn flay(m: &Bound<'_, PyModule>) -> PyResult<()> {
    if let Ok(logger) = pyo3_log::Logger::new(m.py(), pyo3_log::Caching::Loggers) {
        let _ = logger.filter(log::LevelFilter::Trace).install();
    }
    m.add_class::<AnnotationsRemover>()?;
    m.add_class::<FileCollector>()?;
    m.add_class::<ReferencesCounter>()?;
    m.add_class::<NodesRemover>()?;
    m.add_function(wrap_pyfunction!(clear_module_cache, m)?)?;
    m.add_function(wrap_pyfunction!(module_cache_len, m)?)?;
    m.add_function(wrap_pyfunction!(fs_cache_stats, m)?)?;
    Ok(())
}
