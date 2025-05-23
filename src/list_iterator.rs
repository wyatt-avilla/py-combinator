use itertools::Itertools;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyFunction, PyList},
};

trait SizedDoubleEndedIterator: Iterator + DoubleEndedIterator + ExactSizeIterator {}
impl<T> SizedDoubleEndedIterator for T where T: Iterator + DoubleEndedIterator + ExactSizeIterator {}

type PyBaseIteratorT = Box<dyn Iterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PyBaseIterator {
    iter: PyBaseIteratorT,
}

impl PyBaseIterator {
    fn take_inner(&mut self) -> PyBaseIteratorT {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }

    fn to_list<I>(iter: I) -> PyResult<Py<PyList>>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        let v = iter.collect::<PyResult<Vec<_>>>()?;
        Python::with_gil(|py| Ok(PyList::new(py, v)?.unbind()))
    }

    fn filter<I>(
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
    fn map<I>(
        iter: I,
        f: Py<PyFunction>,
    ) -> std::iter::Map<I, impl FnMut(PyResult<Py<PyAny>>) -> PyResult<Py<PyAny>>>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.map(move |x| Python::with_gil(|py| x.and_then(|x| f.call1(py, (x.bind(py),)))))
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    fn fold<I>(mut iter: I, init: Py<PyAny>, f: Py<PyFunction>) -> PyResult<Py<PyAny>>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        Python::with_gil(|py| iter.try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x)))))
    }

    #[allow(clippy::type_complexity)]
    fn enumerate<I>(
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

    fn take<I>(iter: I, n: usize) -> std::iter::Take<I>
    where
        I: Iterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.take(n)
    }
}

type PyDoubleEndedIteratorT =
    Box<dyn DoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>;
#[pyclass]
pub struct PyDoubleEndedIterator {
    iter: PyDoubleEndedIteratorT,
}

impl PyDoubleEndedIterator {
    fn rev<I>(iter: I) -> std::iter::Rev<I>
    where
        I: DoubleEndedIterator<Item = PyResult<Py<PyAny>>>,
    {
        iter.rev()
    }
}

#[pyclass]
pub struct PyExactSizeIteratorWrapper {
    iter: Box<dyn ExactSizeIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>,
}

#[pyclass]
pub struct PySizedDoubleEndedIteratorWrapper {
    iter: Box<dyn SizedDoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>,
}

struct PyListWrapper {
    list: Py<PyList>,
    start: usize,
    end: usize,
}

impl PyListWrapper {
    fn new(list: &Bound<'_, PyList>) -> Self {
        PyListWrapper {
            list: list.clone().unbind(),
            start: 0,
            end: list.len(),
        }
    }
}

impl Iterator for PyListWrapper {
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

impl DoubleEndedIterator for PyListWrapper {
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

impl ExactSizeIterator for PyListWrapper {
    fn len(&self) -> usize {
        self.end - self.start
    }
}

#[pyclass]
pub struct ListIterator {
    iter: Box<dyn SizedDoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>,
}

#[pymethods]
impl ListIterator {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> Self {
        Self {
            iter: Box::new(PyListWrapper::new(list)),
        }
    }
}
