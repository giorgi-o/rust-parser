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
    let mut acc = 0;
    let mut __temp_0 = (a).to_usize(py) * (b).to_usize(py);
    let mut i = 0;
    while (i).to_usize(py) < (10).to_usize(py) {
        acc = (acc).to_usize(py) + (__temp_0).to_usize(py);
        i = (i).to_usize(py) + (1).to_usize(py);
    }

    let mut i = 0;
    while (i).to_usize(py) < (10).to_usize(py) {
        a = (a).to_usize(py) + (1).to_usize(py);
        acc = (acc).to_usize(py) + ((a).to_usize(py) * (b).to_usize(py)).to_usize(py);
        i = (i).to_usize(py) + (1).to_usize(py);
    }

    blackbox(py, (&acc));
    return (py.None()).to_pyany(py);
}
