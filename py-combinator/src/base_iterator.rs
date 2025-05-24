use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyFunction, PyList},
};

type PyBaseIteratorT = Box<dyn Iterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PyBaseIterator {
    iter: PyBaseIteratorT,
}

impl PyBaseIterator {
    pub fn take_inner(&mut self) -> PyBaseIteratorT {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }
}

#[macros::register_methods]
impl PyBaseIterator {
    pub fn new(iter: PyBaseIteratorT) -> Self {
        Self { iter }
    }

    pub fn to_list<I>(iter: I) -> PyResult<Py<PyList>>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        let v = iter.collect::<PyResult<Vec<_>>>()?;
        Python::with_gil(|py| Ok(PyList::new(py, v)?.unbind()))
    }

    pub fn filter<I>(
        iter: I,
        f: Py<PyFunction>,
    ) -> std::iter::Filter<I, impl FnMut(&PyResult<Py<PyAny>>) -> bool>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        let bad_predicate = "exception in filter predicate";

        iter.filter(move |x| {
            Python::with_gil(|py| {
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
        f: Py<PyFunction>,
    ) -> std::iter::Map<I, impl FnMut(PyResult<Py<PyAny>>) -> PyResult<Py<PyAny>>>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.map(move |x| Python::with_gil(|py| x.and_then(|x| f.call1(py, (x.bind(py),)))))
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    pub fn fold<I>(mut iter: I, init: Py<PyAny>, f: Py<PyFunction>) -> PyResult<Py<PyAny>>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        Python::with_gil(|py| iter.try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x)))))
    }

    #[allow(clippy::type_complexity)]
    pub fn enumerate<I>(
        iter: I,
    ) -> std::iter::Map<
        std::iter::Enumerate<I>,
        impl FnMut((usize, Result<Py<PyAny>, PyErr>)) -> Result<Py<PyAny>, PyErr>,
    >
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.enumerate()
            .map(move |(i, x)| Python::with_gil(|py| x.and_then(|x| (i, x).into_py_any(py))))
    }

    pub fn take<I>(iter: I, n: usize) -> std::iter::Take<I>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.take(n)
    }
}
