use pyo3::prelude::*;

type PyDoubleEndedIteratorT =
    Box<dyn DoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PyDoubleEndedIterator {
    iter: PyDoubleEndedIteratorT,
}

impl PyDoubleEndedIterator {
    pub fn new(iter: PyDoubleEndedIteratorT) -> Self {
        Self { iter }
    }
}

#[macros::register_methods(self_generic = S)]
impl crate::double_ended_iterator::PyDoubleEndedIterator {
    #[macros::method_self_arg]
    pub fn take_inner(&mut self) -> PyDoubleEndedIteratorT {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }

    pub fn rev<S>(iter: S) -> std::iter::Rev<S>
    where
        S: DoubleEndedIterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.rev()
    }
}

#[macros::add_trait_methods(PyDoubleEndedIterator, (PyBaseIterator, exclude=(take, enumerate)))]
#[pymethods]
impl PyDoubleEndedIterator {}
