#[pyo3::pyclass]
pub struct PyBaseIterator {
    iter: Box<dyn Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>> + Send + Sync>,
}

impl PyBaseIterator {
    pub fn take_inner(
        &mut self,
    ) -> Box<dyn Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>> + Send + Sync> {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }
}

#[macros::register_methods]
impl crate::base_iterator::PyBaseIterator {
    pub fn new(
        iter: Box<dyn Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>> + Send + Sync>,
    ) -> Self {
        Self { iter }
    }

    pub fn to_list<I>(iter: I) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyList>>
    where
        I: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        let v = iter.collect::<pyo3::PyResult<Vec<_>>>()?;
        pyo3::Python::with_gil(|py| Ok(pyo3::types::PyList::new(py, v)?.unbind()))
    }

    pub fn filter<I>(
        iter: I,
        f: pyo3::Py<pyo3::types::PyFunction>,
    ) -> std::iter::Filter<I, impl FnMut(&pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>) -> bool>
    where
        I: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
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
    pub fn map<I>(
        iter: I,
        f: pyo3::Py<pyo3::types::PyFunction>,
    ) -> std::iter::Map<
        I,
        impl FnMut(
            pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>,
        ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>,
    >
    where
        I: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        iter.map(move |x| pyo3::Python::with_gil(|py| x.and_then(|x| f.call1(py, (x.bind(py),)))))
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    pub fn fold<I>(
        mut iter: I,
        init: pyo3::Py<pyo3::types::PyAny>,
        f: pyo3::Py<pyo3::types::PyFunction>,
    ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>
    where
        I: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        pyo3::Python::with_gil(|py| {
            iter.try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x))))
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn enumerate<I>(
        iter: I,
    ) -> std::iter::Map<
        std::iter::Enumerate<I>,
        impl FnMut(
            (usize, pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>),
        ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>,
    >
    where
        I: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        use pyo3::IntoPyObjectExt;
        iter.enumerate()
            .map(move |(i, x)| pyo3::Python::with_gil(|py| x.and_then(|x| (i, x).into_py_any(py))))
    }

    pub fn take<I>(iter: I, n: usize) -> std::iter::Take<I>
    where
        I: Iterator<Item = pyo3::PyResult<pyo3::Py<pyo3::types::PyAny>>>,
    {
        iter.take(n)
    }
}
