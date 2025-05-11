use std::collections::VecDeque;

use itertools::Itertools;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyFunction, PyInt, PyList, PyTuple},
};

trait PyAnyDoubleEndedIter:
    Iterator<Item = Py<PyAny>> + DoubleEndedIterator + ExactSizeIterator
{
}
impl<T> PyAnyDoubleEndedIter for T where
    T: Iterator<Item = Py<PyAny>> + DoubleEndedIterator + ExactSizeIterator
{
}

type PyListWrapperT = Box<dyn PyAnyDoubleEndedIter + Send + Sync>;

#[pyclass]
pub struct PyListWrapper {
    it: PyListWrapperT,
    to_apply: VecDeque<Function>,
}

enum Function {
    Python(Py<PyFunction>),
    Rust(fn(PyListWrapperT) -> PyListWrapperT),
}

impl PyListWrapper {
    fn apply_all(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> PyResult<PyListWrapperT> {
        let funcs = slf.to_apply.drain(..).collect_vec();
        let mut items = slf.it.by_ref().collect_vec();

        for func in funcs {
            match func {
                Function::Python(f) => {
                    items = items
                        .into_iter()
                        .map(|x| f.call1(py, (x,)))
                        .collect::<Result<Vec<_>, _>>()?;
                }
                Function::Rust(f) => {
                    items = f(Box::new(items.into_iter())).collect();
                }
            }
        }

        Ok(Box::new(items.into_iter()))
    }
}

#[pymethods]
impl PyListWrapper {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> Self {
        PyListWrapper {
            it: Box::new(
                list.iter()
                    .map(pyo3::Bound::unbind)
                    .collect_vec()
                    .into_iter(),
            ),
            to_apply: VecDeque::new(),
        }
    }

    fn map<'a>(mut slf: PyRefMut<'a, Self>, f: Bound<'_, PyFunction>) -> PyRefMut<'a, Self> {
        slf.to_apply.push_back(Function::Python(f.unbind()));
        slf
    }

    fn fold<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        init: Bound<'_, PyAny>,
        f: Bound<'_, PyFunction>,
    ) -> PyResult<Py<PyAny>> {
        let f = f.unbind();

        let folded = PyListWrapper::apply_all(slf, py)?.try_fold(init, |a, x| {
            PyResult::Ok(f.call1(py, (&a, x))?.into_bound(py))
        })?;

        Ok(folded.unbind())
    }

    fn rev(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.to_apply
            .push_back(Function::Rust(|it: PyListWrapperT| -> PyListWrapperT {
                Box::new(it.rev())
            }));
        slf
    }

    fn enumerate(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.to_apply
            .push_back(Function::Rust(|it: PyListWrapperT| -> PyListWrapperT {
                Box::new(it.enumerate().map(|(i, v)| {
                    Python::with_gil(|py| {
                        let i = PyInt::new(py, i).into_py_any(py).unwrap();
                        let tup = PyTuple::new(py, &[i, v]).unwrap().into_py_any(py).unwrap();
                        tup
                    })
                }))
            }));
        slf
    }

    fn to_list<'a>(slf: PyRefMut<'a, Self>, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, PyListWrapper::apply_all(slf, py)?.collect_vec())
    }

    #[getter]
    fn uncalled(&self) -> usize {
        self.to_apply.len()
    }
}
