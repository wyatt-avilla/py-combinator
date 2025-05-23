use pyo3::{prelude::*, types::PyList};

#[pyclass]
pub struct PyListIterator {
    list: Py<PyList>,
    start: usize,
    end: usize,
}

impl PyListIterator {
    pub fn new(list: &Bound<'_, PyList>) -> Self {
        PyListIterator {
            list: list.clone().unbind(),
            start: 0,
            end: list.len(),
        }
    }
}

impl Iterator for PyListIterator {
    type Item = PyResult<Py<PyAny>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            Python::with_gil(|py| -> Option<PyResult<Py<PyAny>>> {
                let item = self
                    .list
                    .bind(py)
                    .get_item(self.start)
                    .map(pyo3::Bound::unbind);
                self.start += 1;
                Some(item)
            })
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for PyListIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            Python::with_gil(|py| -> Option<PyResult<Py<PyAny>>> {
                self.end -= 1;
                let item = self
                    .list
                    .bind(py)
                    .get_item(self.end)
                    .map(pyo3::Bound::unbind);
                Some(item)
            })
        } else {
            None
        }
    }
}

impl ExactSizeIterator for PyListIterator {
    fn len(&self) -> usize {
        self.end - self.start
    }
}
