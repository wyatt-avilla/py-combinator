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
}

impl crate::base_iterator::PyBaseIterator {
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

    #[allow(clippy::type_complexity)]
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

    pub fn take<S>(iter: S, n: usize) -> std::iter::Take<S>
    where
        S: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        iter.take(n)
    }
}
