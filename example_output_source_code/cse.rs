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
    let mut b = blackbox(py, (&2));
    let mut c = (a).to_usize(py) + (b).to_usize(py);
    let mut d = c;
    blackbox(py, (&c));
    blackbox(py, (&d));
    let mut e = d;
    a = 9;
    let mut f = (a).to_usize(py) + (b).to_usize(py);
    blackbox(py, (&e));
    blackbox(py, (&f));
    return (py.None()).to_pyany(py);
}
