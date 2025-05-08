use std::collections::VecDeque;

use itertools::Itertools;
use pyo3::{
    prelude::*,
    types::{PyFunction, PyList},
};

type AnyIteratorT = Box<dyn Iterator<Item = Py<PyAny>> + Send + Sync>;

#[pyclass]
struct AnyIterator {
    it: AnyIteratorT,
    to_apply: VecDeque<Py<PyFunction>>,
}

impl AnyIterator {
    fn apply_all(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> PyResult<AnyIteratorT> {
        let funcs = slf.to_apply.drain(0..).collect_vec();
        let mapped = slf
            .it
            .by_ref()
            .map(|x| funcs.iter().try_fold(x, |acc, f| f.call1(py, (&acc,))))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Box::new(mapped.into_iter()))
    }
}

#[pymethods]
impl AnyIterator {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> Self {
        AnyIterator {
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
        slf.to_apply.push_back(f.unbind());
        slf
    }

    fn fold<'a>(
        slf: PyRefMut<'a, Self>,
        py: Python<'a>,
        init: Bound<'_, PyAny>,
        f: Bound<'_, PyFunction>,
    ) -> PyResult<Py<PyAny>> {
        let f = f.unbind();

        let folded = AnyIterator::apply_all(slf, py)?.try_fold(init, |a, x| {
            PyResult::Ok(f.call1(py, (&a, x))?.into_bound(py))
        })?;

        Ok(folded.unbind())
    }

    fn to_list<'a>(slf: PyRefMut<'a, Self>, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, AnyIterator::apply_all(slf, py)?.collect_vec())
    }

    #[getter]
    fn uncalled(&self) -> usize {
        self.to_apply.len()
    }
}

#[pymodule]
#[allow(clippy::unnecessary_wraps)]
fn _py_combinator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m.add_class::<AnyIterator>();
    Ok(())
}
