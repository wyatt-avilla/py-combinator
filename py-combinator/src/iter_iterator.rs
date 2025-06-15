use pyo3::{prelude::*, types::PyIterator};

#[pyclass]
pub struct PyIterIterator {
    iter: Py<PyIterator>,
}

impl PyIterIterator {
    pub fn new(iter: &Bound<'_, PyIterator>) -> Self {
        PyIterIterator {
            iter: iter.clone().unbind(),
        }
    }
}

impl Iterator for PyIterIterator {
    type Item = PyResult<Py<PyAny>>;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| Some(self.iter.call_method0(py, "next")))
    }
}
