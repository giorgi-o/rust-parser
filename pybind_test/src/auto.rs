use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::util::*;

#[pymodule]
fn DataManagement(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Buffer>()?;

    m.add_function(wrap_pyfunction!(allocateMemory, m)?)?;
    m.add_function(wrap_pyfunction!(freeMemory, m)?)?;
    m.add_function(wrap_pyfunction!(optimizeMe, m)?)?;
    m.add_function(wrap_pyfunction!(processStream, m)?)?;

    Ok(())
}

#[pyfunction]
fn allocateMemory(py: Python<'_>, size: Py<PyAny>) -> Py<PyAny> {
    let buffer = allocate(py, &size);
    return (buffer).to_pyany(py);
}

#[pyfunction]
fn freeMemory(py: Python<'_>, ptr: Py<PyAny>) -> Py<PyAny> {
    free(py, &ptr);
    return (10).to_pyany(py);
}

#[pyfunction]
fn optimizeMe(py: Python<'_>) -> Py<PyAny> {
    let sum = 3;
    allocate(py, &sum);
    return (py.None()).to_pyany(py);
}

#[pyfunction]
fn processStream(py: Python<'_>, streamSize: Py<PyAny>) -> Py<PyAny> {
    let blocksize = 10;
    let streamPtr = allocate(py, &streamSize);
    let blocks = PyList::new(py, Vec::<Buffer>::new()).unwrap().unbind();
    let mut i = 0;
    while (i).to_usize(py) < (streamSize).to_usize(py) {
        let blockPtr = (streamPtr).to_buffer(py).borrow(py, &blocksize, &i);
        blocks
            .call_method(py, "append", ((&blockPtr),), None)
            .unwrap();
        i = (i).to_usize(py) + (blocksize).to_usize(py);
    }

    return (blocks).to_pyany(py);
}
