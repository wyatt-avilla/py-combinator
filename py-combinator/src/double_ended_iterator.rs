use itertools::Itertools;
use pyo3::{
    prelude::*,
    types::{PyFunction, PyList},
};

use crate::base_iterator::PyBaseIterator;

type PyDoubleEndedIteratorT =
    Box<dyn DoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PyDoubleEndedIterator {
    iter: PyDoubleEndedIteratorT,
}

impl PyDoubleEndedIterator {
    pub fn rev<I>(iter: I) -> std::iter::Rev<I>
    where
        I: DoubleEndedIterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.rev()
    }
}

impl PyDoubleEndedIterator {
    pub fn new(iter: PyDoubleEndedIteratorT) -> Self {
        Self { iter }
    }

    pub fn take_inner(&mut self) -> PyDoubleEndedIteratorT {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }
}

#[pymethods]
impl PyDoubleEndedIterator {
    pub fn to_list(&mut self) -> PyResult<Py<PyList>> {
        PyBaseIterator::to_list(&mut self.iter)
    }

    pub fn map(&mut self, f: Py<PyFunction>) -> Self {
        Self::new(Box::new(PyBaseIterator::map(self.take_inner(), f)))
    }

    pub fn filter(&mut self, f: Py<PyFunction>) -> Self {
        PyDoubleEndedIterator::new(Box::new(PyBaseIterator::filter(self.take_inner(), f)))
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    pub fn fold(&mut self, init: Py<PyAny>, f: Py<PyFunction>) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            self.take_inner()
                .try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x))))
        })
    }

    pub fn take(&mut self, n: usize) -> Self {
        Self::new(Box::new(
            PyBaseIterator::take(self.iter.by_ref(), n)
                .collect_vec()
                .into_iter(),
        ))
    }
}
