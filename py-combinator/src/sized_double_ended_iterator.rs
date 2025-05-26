use itertools::Itertools;
use pyo3::{
    prelude::*,
    types::{PyFunction, PyList},
};

use crate::{base_iterator::PyBaseIterator, double_ended_iterator::PyDoubleEndedIterator};

pub trait SizedDoubleEndedIterator: Iterator + DoubleEndedIterator + ExactSizeIterator {}
impl<T> SizedDoubleEndedIterator for T where T: Iterator + DoubleEndedIterator + ExactSizeIterator {}

type PySizedDoubleEndedIteratorT =
    Box<dyn SizedDoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PySizedDoubleEndedIterator {
    iter: PySizedDoubleEndedIteratorT,
}

impl PySizedDoubleEndedIterator {
    pub fn new(iter: PySizedDoubleEndedIteratorT) -> Self {
        Self { iter }
    }
}

#[macros::register_methods(self_generic = S)]
impl crate::sized_double_ended_iterator::PySizedDoubleEndedIterator {
    #[macros::method_self_arg]
    pub fn take_inner(&mut self) -> PySizedDoubleEndedIteratorT {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }
}

#[pymethods]
impl PySizedDoubleEndedIterator {
    pub fn to_list(&mut self) -> PyResult<Py<PyList>> {
        PyBaseIterator::to_list(&mut self.iter)
    }

    pub fn map(&mut self, f: Py<PyFunction>) -> Self {
        Self::new(Box::new(PyBaseIterator::map(self.take_inner(), f)))
    }

    pub fn filter(&mut self, f: Py<PyFunction>) -> PyDoubleEndedIterator {
        PyDoubleEndedIterator::new(Box::new(PyBaseIterator::filter(self.take_inner(), f)))
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    pub fn fold(&mut self, init: Py<PyAny>, f: Py<PyFunction>) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            self.take_inner()
                .try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x))))
        })
    }

    pub fn enumerate(&mut self) -> Self {
        Self::new(Box::new(PyBaseIterator::enumerate(self.take_inner())))
    }

    pub fn take(&mut self, n: usize) -> Self {
        Self::new(Box::new(
            PyBaseIterator::take(self.iter.by_ref(), n)
                .collect_vec()
                .into_iter(),
        ))
    }

    pub fn rev(&mut self) -> Self {
        Self::new(Box::new(PyDoubleEndedIterator::rev(self.take_inner())))
    }
}
