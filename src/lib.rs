use std::collections::VecDeque;

use pyo3::{
    prelude::*,
    types::{PyFunction, PyInt, PyList},
};

enum SupportedIterableTypes {
    PyInt(pyo3::types::PyInt),
}

// TODO: generate with macro based on `SupportedIterableTypes`
#[pyclass]
struct IteratorOfInt {
    it: Box<dyn Iterator<Item = Py<PyInt>> + Send + Sync>,
    to_apply: VecDeque<Py<PyFunction>>,
}

#[pymethods]
impl IteratorOfInt {
    #[new]
    fn py_new(list: &Bound<'_, PyList>) -> PyResult<Self> {
        let ints = list
            .iter()
            .map(|x| x.downcast_into::<PyInt>().map(pyo3::Bound::unbind))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(IteratorOfInt {
            it: Box::new(ints.into_iter()),
            to_apply: VecDeque::new(),
        })
    }

    fn map<'a>(mut slf: PyRefMut<'a, Self>, f: Bound<'_, PyFunction>) -> PyRefMut<'a, Self> {
        slf.to_apply.push_back(f.unbind());
        slf
    }

    #[getter]
    fn uncalled(&self) -> usize {
        self.to_apply.len()
    }
}

#[pymodule]
fn _py_combinator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m.add_class::<IteratorOfInt>();
    Ok(())
}
