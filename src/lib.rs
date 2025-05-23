#![warn(clippy::pedantic)]

mod base_iterator;
mod double_ended_iterator;
mod exact_size_iterator;
mod list_iterator;
mod sized_double_ended_iterator;

use pyo3::{IntoPyObjectExt, exceptions::PyTypeError, prelude::*, types::PyList};

#[pyfunction]
fn iterator_from(iterable: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        if let Ok(list) = iterable.downcast::<PyList>() {
            let list_iter = list_iterator::PyListIterator::new(list);
            sized_double_ended_iterator::PySizedDoubleEndedIterator::new(Box::new(list_iter))
                .into_py_any(py)
        } else {
            Err(PyTypeError::new_err(format!(
                "Cannot construct iterator from type {}",
                iterable.get_type().name()?
            )))
        }
    })
}

#[pymodule]
#[allow(clippy::unnecessary_wraps)]
fn _py_combinator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m.add_class::<sized_double_ended_iterator::PySizedDoubleEndedIterator>();
    let _ = m.add_function(wrap_pyfunction!(iterator_from, m)?);
    Ok(())
}
