use std::collections::VecDeque;

use itertools::Itertools;
use pyo3::{
    prelude::*,
    types::{PyFunction, PyList},
};

enum SupportedIterableTypes {
    PyInt(pyo3::types::PyInt),
}

// TODO: generate with macro based on `SupportedIterableTypes`
#[pyclass]
struct AnyIterator {
    it: Box<dyn Iterator<Item = Py<PyAny>> + Send + Sync>,
    to_apply: VecDeque<Py<PyFunction>>,
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

    fn to_list<'a>(mut slf: PyRefMut<'a, Self>, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        PyList::new(py, slf.it.by_ref().collect_vec())
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
