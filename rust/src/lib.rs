mod bundle;
mod common;
mod rustpython;
use bundle::file_collector::FileCollector;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_flay_rs")]
mod flay {
    use super::*;

    #[pymodule]
    mod bundle {
        #[pymodule_export]
        use super::FileCollector;
        use super::*;

        #[pymodule_init]
        fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
            Python::with_gil(|py| {
                let _ = py
                    .import("sys")?
                    .getattr("modules")?
                    .set_item("flay._flay_rs.bundle", m);
                Ok(())
            })
        }
    }
}
