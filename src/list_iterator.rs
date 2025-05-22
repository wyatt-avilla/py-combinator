use itertools::Itertools;
use pyo3::{
    IntoPyObjectExt, PyClass,
    prelude::*,
    types::{PyBool, PyFunction, PyList},
};

trait SizedDoubleEndedIterator: Iterator + DoubleEndedIterator + ExactSizeIterator {}
impl<T> SizedDoubleEndedIterator for T where T: Iterator + DoubleEndedIterator + ExactSizeIterator {}

impl<T> PyBaseIterator for T where T: Iterator<Item = PyResult<Py<PyAny>>> + Send + Sync {}
trait PyBaseIterator: Iterator<Item = PyResult<Py<PyAny>>> + Send + Sync {
    fn to_list(&mut self) -> PyResult<Py<PyList>> {
        let v = self.collect::<PyResult<Vec<_>>>()?;
        Python::with_gil(|py| Ok(PyList::new(py, v)?.unbind()))
    }

    fn filter(&mut self, f: Py<PyFunction>) -> Box<dyn PyBaseIterator + '_> {
        let bad_predicate = "exception in filter predicate";

        Box::new(std::iter::Iterator::filter(self, move |x| {
            Python::with_gil(|py| {
                x.as_ref()
                    .map(|x| f.call1(py, (x.bind(py),)))
                    .expect(bad_predicate)
                    .map(|x| x.is_truthy(py))
                    .and_then(|x| x)
                    .expect(bad_predicate)
            })
        }))
    }
}

#[pyclass]
pub struct PyBaseIteratorWrapper {
    iter: Box<dyn PyBaseIterator>,
}

impl PyBaseIteratorWrapper {
    fn take_inner(&mut self) -> Box<dyn PyBaseIterator> {
        std::mem::replace(&mut self.iter, Box::new(std::iter::empty()))
    }
}

#[pyclass]
pub struct PyDoubleEndedIteratorWrapper {
    iter: Box<dyn DoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>,
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

    fn map(mut slf: PyRefMut<'_, Self>, f: Py<PyFunction>) -> PySizedDoubleEndedIteratorWrapper {
        let it = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        PySizedDoubleEndedIteratorWrapper {
            iter: Box::new(
                it.map(move |x| Python::with_gil(|py| x.and_then(|x| f.call1(py, (x.bind(py),))))),
            ),
        }
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    fn fold(
        mut slf: PyRefMut<'_, Self>,
        init: Py<PyAny>,
        f: Py<PyFunction>,
    ) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            slf.iter
                .by_ref()
                .try_fold(init, |a, x| x.and_then(|x| f.call1(py, (&a, x))))
        })
    }

    fn rev(mut slf: PyRefMut<'_, Self>) -> PySizedDoubleEndedIteratorWrapper {
        let it = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        PySizedDoubleEndedIteratorWrapper {
            iter: Box::new(it.rev()),
        }
    }

    fn take(mut slf: PyRefMut<'_, Self>, n: usize) -> Self {
        Self {
            iter: Box::new(slf.iter.by_ref().take(n).collect_vec().into_iter()),
        }
    }

    fn enumerate(mut slf: PyRefMut<'_, Self>) -> PySizedDoubleEndedIteratorWrapper {
        let it = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        PySizedDoubleEndedIteratorWrapper {
            iter: Box::new(
                it.enumerate().map(move |(i, x)| {
                    Python::with_gil(|py| x.and_then(|x| (i, x).into_py_any(py)))
                }),
            ),
        }
    }

    fn filter(mut slf: PyRefMut<'_, Self>, f: Py<PyFunction>) -> PyDoubleEndedIteratorWrapper {
        let it = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        let bad_predicate = "exception in filter predicate";

        PyDoubleEndedIteratorWrapper {
            iter: Box::new(it.filter(move |x| {
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
            })),
        }
    }

    fn to_list(&mut self) -> PyResult<Py<PyList>> {
        PyBaseIterator::to_list(&mut self.iter)
    }
}
