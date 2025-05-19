use itertools::Itertools;
use pyo3::{
    prelude::*,
    types::{PyBool, PyFunction, PyList},
};

#[pyclass]
pub struct ListIterator {
    iter: Box<dyn DoubleEndedIterator<Item = PyResult<Py<PyAny>>> + Send + Sync>,
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

#[pymethods]
impl ListIterator {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> Self {
        ListIterator {
            iter: Box::new(PyListWrapper::new(list)),
        }
    }

    fn map(mut slf: PyRefMut<'_, Self>, f: Py<PyFunction>) -> PyRefMut<'_, Self> {
        let replaced = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        slf.iter = Box::new(
            replaced
                .map(move |x| Python::with_gil(|py| x.and_then(|x| f.call1(py, (x.bind(py),))))),
        );

        slf
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

    fn rev(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        let replaced = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        slf.iter = Box::new(replaced.rev());

        slf
    }

    fn take(slf: PyRefMut<'_, Self>, n: usize) -> PyResult<PyRefMut<'_, Self>> {
        todo!()
    }

    fn enumerate(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        let replaced = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        todo!();
        //slf.iter = Box::new(
        //    replaced
        //        .enumerate()
        //        .map(move |(i, x)| Python::with_gil(|py| x.and_then(|x| (i, x).into_py_any(py)))),
        //);

        slf
    }

    fn filter(mut slf: PyRefMut<'_, Self>, f: Py<PyFunction>) -> PyRefMut<'_, Self> {
        let replaced = std::mem::replace(&mut slf.iter, Box::new(std::iter::empty()));

        slf.iter = Box::new(replaced.filter(move |x| {
            Python::with_gil(|py| {
                // TODO
                let p = x
                    .as_ref()
                    .map(|x| f.call1(py, (x.bind(py),)))
                    .unwrap()
                    .map(|x| x.downcast_bound::<PyBool>(py).unwrap().is_true())
                    .unwrap();

                p
            })
        }));

        slf
    }

    fn to_list<'a>(mut slf: PyRefMut<'a, Self>, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, slf.iter.by_ref().collect::<PyResult<Vec<_>>>()?)
    }
}
