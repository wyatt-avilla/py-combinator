#![warn(clippy::pedantic)]

mod base_iterator;
mod double_ended_iterator;
mod exact_size_iterator;
mod iter_iterator;
mod list_iterator;
mod sized_double_ended_iterator;

mod iterators {
    #[allow(unused_imports)]
    pub use crate::{
        base_iterator::PyBaseIterator, double_ended_iterator::PyDoubleEndedIterator,
        exact_size_iterator::PyExactSizeIterator,
        sized_double_ended_iterator::PySizedDoubleEndedIterator,
    };
}

use pyo3::{IntoPyObjectExt, exceptions::PyTypeError, prelude::*, types::PyList};

#[pyfunction]
fn iterator_from(iterable: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        if let Ok(list) = iterable.downcast::<PyList>() {
            let list_iter = list_iterator::PyListIterator::new(list);
            sized_double_ended_iterator::PySizedDoubleEndedIterator::new(Box::new(list_iter))
                .into_py_any(py)
        } else {
            match iterable.try_iter() {
                Ok(it) => base_iterator::PyBaseIterator::new(Box::new(
                    iter_iterator::PyIterIterator::new(&it),
                ))
                .into_py_any(py),
                Err(e) => Err(PyTypeError::new_err(format!(
                    "Cannot construct iterator from type {} ({})",
                    e,
                    iterable.get_type().name()?,
                ))),
            }
        }
    })
}

#[pymodule]
#[allow(clippy::unnecessary_wraps)]
fn _py_combinator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<base_iterator::PyBaseIterator>()?;
    m.add_class::<exact_size_iterator::PyExactSizeIterator>()?;
    m.add_class::<double_ended_iterator::PyDoubleEndedIterator>()?;
    m.add_class::<sized_double_ended_iterator::PySizedDoubleEndedIterator>()?;
    let _ = m.add_function(wrap_pyfunction!(iterator_from, m)?);
    Ok(())
}
