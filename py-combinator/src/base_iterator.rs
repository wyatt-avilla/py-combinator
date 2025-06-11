#[pyo3::pyclass]
pub struct PyBaseIterator {
    iter: Box<dyn Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>> + Send + Sync>,
}

impl PyBaseIterator {
    pub fn new(
        iter: Box<dyn Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>> + Send + Sync>,
    ) -> Self {
        Self { iter }
    }
}

#[macros::register_methods(self_generic = S)]
impl crate::base_iterator::PyBaseIterator {
    #[macros::method_self_arg]
    pub fn take_inner(
        &mut self,
    ) -> Box<dyn Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>> + Send + Sync> {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }

    #[doc = "Converts the iterator to a list"]
    #[macros::return_literal]
    pub fn to_list<S>(iter: S) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyList>>
    where
        S: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        let v = iter.collect::<pyo3::PyResult<Vec<_>>>()?;
        pyo3::Python::with_gil(|py| Ok(pyo3::types::PyList::new(py, v)?.unbind()))
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    #[macros::return_literal]
    #[doc = "Folds every element into an accumulator by repeatedly applying `f`.
             
             Examples:
                 iter # [2, 4, 6]
                 iter.fold(1, lambda a, x: a * x) # 48"]
    pub fn fold<S>(
        mut iter: S,
        init: pyo3::Py<pyo3::types::PyAny>,
        f: pyo3::Py<pyo3::types::PyFunction>,
    ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>
    where
        S: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        pyo3::Python::with_gil(|py| {
            iter.try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x))))
        })
    }

    #[allow(clippy::type_complexity)]
    #[doc = "Creates a new iterator by applying `f` to each element.
             
             Examples:
                 iter # [1, 2, 3]
                 iter.map(lambda x: x + 1) # [2, 3, 4]"]
    pub fn map<S>(
        iter: S,
        f: pyo3::Py<pyo3::types::PyFunction>,
    ) -> std::iter::Map<
        S,
        impl FnMut(
            pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>,
        ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>,
    >
    where
        S: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        iter.map(move |x| pyo3::Python::with_gil(|py| x.and_then(|x| f.call1(py, (x.bind(py),)))))
    }

    #[doc = "Creates a new iterator that yields elements for which `f` returns `true`.
             
             Examples:
                 iter # [1, 2, 3]
                 iter.filter(lambda x: x % 2 == 0) # [2]"]
    #[macros::strips_traits(PyExactSizeIterator)]
    pub fn filter<S>(
        iter: S,
        f: pyo3::Py<pyo3::types::PyFunction>,
    ) -> std::iter::Filter<S, impl FnMut(&pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>) -> bool>
    where
        S: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        let bad_predicate = "exception in filter predicate";

        iter.filter(move |x| {
            pyo3::Python::with_gil(|py| {
                let p = x
                    .as_ref()
                    .map(|x| f.call1(py, (x.bind(py),)))
                    .expect(bad_predicate)
                    .map(|x| x.is_truthy(py))
                    .and_then(|x| x)
                    .expect(bad_predicate);

                p
            })
        })
    }

    #[doc = "The iterator returned yields pairs `(i, val)`, where `i` is the
             current index of iteration and `val` is the value returned by the
             iterator.
             
             Examples:
                 iter # [4, 9, 16]
                 iter.enumerate() # [(0, 4), (1, 9), (2, 16)]"]
    #[allow(clippy::type_complexity)]
    pub fn enumerate<S>(
        iter: S,
    ) -> std::iter::Map<
        std::iter::Enumerate<S>,
        impl FnMut(
            (usize, pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>),
        ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>,
    >
    where
        S: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        use pyo3::IntoPyObjectExt;
        iter.enumerate()
            .map(move |(i, x)| pyo3::Python::with_gil(|py| x.and_then(|x| (i, x).into_py_any(py))))
    }
}

#[macros::add_trait_methods(PyBaseIterator)]
#[pyo3::pymethods]
impl PyBaseIterator {
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
