#![warn(clippy::pedantic)]

mod list_iterator;

use pyo3::prelude::*;

#[pymodule]
#[allow(clippy::unnecessary_wraps)]
fn _py_combinator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m.add_class::<list_iterator::ListIterator>();
    Ok(())
}
