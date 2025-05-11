use pyo3::{prelude::*, types::PyIterator};

#[pyclass]
pub struct PyIterWrapper {
    _it: Py<PyIterator>,
}

#[pymethods]
impl PyIterWrapper {
    #[new]
    fn py_new(py_iterator: &Bound<'_, PyIterator>) -> Self {
        PyIterWrapper {
            _it: py_iterator.clone().unbind(),
        }
    }
}
