use std::collections::VecDeque;

use itertools::Itertools;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyBool, PyFunction, PyList},
};

#[pyclass]
pub struct PyListWrapper {
    list: Py<PyList>,
    to_apply: VecDeque<Operation>,
}

enum Operation {
    Function(Function),
    ListTransform(ListTransformFunctionSignature),
}

enum Function {
    Python(Py<PyFunction>),
    Rust(RustFunctionSignature),
}

type ListTransformFunctionSignature = Box<dyn Fn(Bound<'_, PyList>) -> PyResult<()> + Send + Sync>;

type RustFunctionSignature =
    Box<dyn Fn(Python<'_>, Py<PyAny>) -> PyResult<Py<PyAny>> + Send + Sync>;

impl PyListWrapper {
    fn apply_all<'p>(mut slf: PyRefMut<'_, Self>, py: Python<'p>) -> PyResult<Bound<'p, PyList>> {
        let ops = slf.to_apply.drain(..).collect_vec();
        let items = slf.list.clone_ref(py).into_bound(py);

        for op in ops {
            match op {
                Operation::ListTransform(f) => f(items.clone())?,
                Operation::Function(func) => {
                    for i in 0..items.len() {
                        let item = items.get_item(i)?;
                        let new_item = match &func {
                            Function::Python(f) => f.call1(py, (item,))?,
                            Function::Rust(f) => f(py, item.unbind())?,
                        };
                        items.set_item(i, new_item)?;
                    }
                }
            }
        }

        Ok(items)
    }
}

#[pymethods]
impl PyListWrapper {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> Self {
        PyListWrapper {
            list: list.clone().unbind(),
            to_apply: VecDeque::new(),
        }
    }

    fn map<'a>(mut slf: PyRefMut<'a, Self>, f: Bound<'_, PyFunction>) -> PyRefMut<'a, Self> {
        slf.to_apply
            .push_back(Operation::Function(Function::Python(f.unbind())));
        slf
    }

    fn fold<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        init: Bound<'_, PyAny>,
        f: Bound<'_, PyFunction>,
    ) -> PyResult<Py<PyAny>> {
        let f = f.unbind();

        let folded = PyListWrapper::apply_all(slf, py)?
            .iter()
            .try_fold(init, |a, x| {
                PyResult::Ok(f.call1(py, (&a, x))?.into_bound(py))
            })?;

        Ok(folded.unbind())
    }

    fn rev(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.to_apply.push_back(Operation::ListTransform(Box::new(
            |list: Bound<'_, PyList>| list.reverse(),
        )));
        slf
    }

    fn enumerate(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        let counter = std::sync::atomic::AtomicUsize::new(0);

        let func = move |py: Python<'_>, py_any: Py<PyAny>| -> PyResult<Py<PyAny>> {
            let current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            (current, py_any).into_py_any(py)
        };

        let boxed_func = Box::new(func) as RustFunctionSignature;

        slf.to_apply
            .push_back(Operation::Function(Function::Rust(boxed_func)));

        slf
    }

    fn filter(mut slf: PyRefMut<'_, Self>, f: Py<PyFunction>) -> PyRefMut<'_, Self> {
        slf.to_apply.push_back(Operation::ListTransform(Box::new(
            move |list: Bound<'_, PyList>| {
                Python::with_gil(|py| {
                    let mut delete_idxs: Vec<(usize, usize)> = Vec::new();
                    for i in 0..list.len() {
                        let item = list.get_item(i)?;
                        let keep = f
                            .call1(py, (item,))?
                            .downcast_bound::<PyBool>(py)?
                            .is_true();

                        if !keep {
                            match delete_idxs.last_mut() {
                                Some(t) if t.1 == i - 1 => t.1 = i,
                                _ => delete_idxs.push((i, i)),
                            }
                        }
                    }

                    for tup in delete_idxs.into_iter().rev() {
                        if tup.0 == tup.1 {
                            list.del_item(tup.0)?;
                        } else {
                            list.del_slice(tup.0, tup.1 + 1)?;
                        }
                    }

                    Ok(())
                })
            },
        )));
        slf
    }

    fn to_list<'a>(slf: PyRefMut<'a, Self>, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyListWrapper::apply_all(slf, py)
    }

    #[getter]
    fn uncalled(&self) -> usize {
        self.to_apply.len()
    }
}
