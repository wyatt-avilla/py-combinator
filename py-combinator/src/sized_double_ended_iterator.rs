use pyo3::prelude::*;

pub trait SizedDoubleEndedIterator: Iterator + DoubleEndedIterator + ExactSizeIterator {}
impl<T> SizedDoubleEndedIterator for T where T: Iterator + DoubleEndedIterator + ExactSizeIterator {}

type PySizedDoubleEndedIteratorT =
    Box<dyn SizedDoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyo3::pyclass(generic)]
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

#[macros::add_trait_methods(
    PySizedDoubleEndedIterator,
    PyBaseIterator,
    PyDoubleEndedIterator,
    PyExactSizeIterator
)]
#[pymethods]
impl PySizedDoubleEndedIterator {
    #[doc = "Consumes the first `n` elements of the iterator.
             
             Examples:
                 iter # [4, 9, 16]
                 iter.take(2) # [4, 9]"]
    pub fn take(&mut self, n: usize) -> Self {
        Self::new(Box::new(
            self.iter.by_ref().take(n).collect::<Vec<_>>().into_iter(),
        ))
    }
}
