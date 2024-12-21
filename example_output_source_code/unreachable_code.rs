use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::util::*;

#[pymodule]
fn ExampleRegion(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Buffer>()?;

    m.add_function(wrap_pyfunction!(main, m)?)?;

    Ok(())
}

#[pyfunction]
fn main(py: Python<'_>) -> Py<PyAny> {
    let mut a = blackbox(py, (&1));
    blackbox(py, (&a));
    return (a).to_pyany(py);
}
