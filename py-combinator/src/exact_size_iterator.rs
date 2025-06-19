use pyo3::prelude::*;

type PyExactSizeIteratorT = Box<dyn ExactSizeIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyo3::pyclass(generic)]
pub struct PyExactSizeIterator {
    iter: PyExactSizeIteratorT,
}

impl PyExactSizeIterator {
    pub fn new(iter: PyExactSizeIteratorT) -> Self {
        Self { iter }
    }
}

#[macros::register_methods(self_generic = S)]
impl crate::exact_size_iterator::PyExactSizeIterator {
    #[macros::method_self_arg]
    pub fn take_inner(&mut self) -> PyExactSizeIteratorT {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }
}

#[macros::add_trait_methods(PyExactSizeIterator, PyBaseIterator)]
#[pymethods]
impl PyExactSizeIterator {
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
