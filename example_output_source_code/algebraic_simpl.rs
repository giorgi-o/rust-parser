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
    let mut a = 1;
    blackbox(py, (&a));
    let mut a1 = a;
    blackbox(py, (&a1));
    let mut b = 2;
    blackbox(py, (&b));
    let mut b1 = b;
    blackbox(py, (&b1));
    return (py.None()).to_pyany(py);
}
