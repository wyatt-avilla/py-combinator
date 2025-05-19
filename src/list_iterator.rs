use std::collections::VecDeque;

use itertools::Itertools;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyBool, PyFunction, PyList},
};

#[pyclass]
pub struct ListIterator {
    list: Py<PyList>,
    to_apply: VecDeque<Operation>,
}

enum Operation {
    ElementTransform(Function),
    ListTransform(ListTransformFunctionSignature),
}

enum Function {
    Python(Py<PyFunction>),
    Rust(RustFunctionSignature),
}

type ListTransformFunctionSignature = Box<dyn Fn(Bound<'_, PyList>) -> PyResult<()> + Send + Sync>;

type RustFunctionSignature =
    Box<dyn Fn(Python<'_>, Py<PyAny>) -> PyResult<Py<PyAny>> + Send + Sync>;

impl ListIterator {
    fn apply_all<'p>(mut slf: PyRefMut<'_, Self>, py: Python<'p>) -> PyResult<Bound<'p, PyList>> {
        let ops = slf.to_apply.drain(..).collect_vec();
        let items = slf.list.clone_ref(py).into_bound(py);

        for op in ops {
            match op {
                Operation::ListTransform(f) => f(items.clone())?,
                Operation::ElementTransform(func) => {
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
impl ListIterator {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> Self {
        ListIterator {
            list: list.clone().unbind(),
            to_apply: VecDeque::new(),
        }
    }

    fn map<'a>(mut slf: PyRefMut<'a, Self>, f: Bound<'_, PyFunction>) -> PyRefMut<'a, Self> {
        slf.to_apply
            .push_back(Operation::ElementTransform(Function::Python(f.unbind())));
        slf
    }

    #[allow(clippy::needless_pass_by_value)] // for f
    fn fold(slf: PyRefMut<'_, Self>, init: Py<PyAny>, f: Py<PyFunction>) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            ListIterator::apply_all(slf, py)?
                .into_iter()
                .try_fold(init, |a, x| f.call1(py, (&a, x)))
        })
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
            .push_back(Operation::ElementTransform(Function::Rust(boxed_func)));

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

                    delete_idxs
                        .into_iter()
                        .rev()
                        .try_for_each(|(l, r)| list.del_slice(l, r + 1))
                })
            },
        )));
        slf
    }

    fn to_list<'a>(slf: PyRefMut<'a, Self>, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        ListIterator::apply_all(slf, py)
    }

    #[getter]
    fn uncalled(&self) -> usize {
        self.to_apply.len()
    }
}
