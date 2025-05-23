use pyo3::prelude::*;

type PyExactSizeIteratorT = Box<dyn ExactSizeIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PyExactSizeIterator {
    iter: PyExactSizeIteratorT,
}
