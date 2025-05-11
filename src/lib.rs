mod pyiter_wrapper;
mod pylist_wrapper;

use pyo3::prelude::*;

#[pymodule]
#[allow(clippy::unnecessary_wraps)]
fn _py_combinator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m.add_class::<pylist_wrapper::PyListWrapper>();
    let _ = m.add_class::<pyiter_wrapper::PyIterWrapper>();
    Ok(())
}
