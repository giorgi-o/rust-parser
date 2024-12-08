use std::sync::{Arc, RwLock};

use pyo3::exceptions::PyIndexError;
use pyo3::types::PyList;

use pyo3::{prelude::*, IntoPyObjectExt};

#[pymodule]
        fn DataManagement(m: &Bound<'_, PyModule>) -> PyResult<()> {
            m.add_class::<Buffer>()?;

            m.add_function(wrap_pyfunction!(allocateMemory, m)?)?;
m.add_function(wrap_pyfunction!(freeMemory, m)?)?;
m.add_function(wrap_pyfunction!(processStream, m)?)?;

            Ok(())
        }
        
        
            #[pyfunction]
fn allocateMemory(py: Python<'_>, size: Py<PyAny>) -> Py<PyAny> {
    let mut buffer = (allocate(py, (size).clone1(py).into_py_any(py).unwrap())).clone1(py).into_py_any(py).unwrap();
return (buffer).clone1(py).into_py_any(py).unwrap();
    return py.None();
}

            #[pyfunction]
fn freeMemory(py: Python<'_>, ptr: Py<PyAny>) -> Py<PyAny> {
    free(py, (ptr).clone1(py).into_py_any(py).unwrap());
return (10).clone1(py).into_py_any(py).unwrap();
    return py.None();
}

            #[pyfunction]
fn processStream(py: Python<'_>, streamSize: Py<PyAny>) -> Py<PyAny> {
    let mut blocksize = (10).clone1(py).into_py_any(py).unwrap();
let mut streamPtr = (allocate(py, (streamSize).clone1(py).into_py_any(py).unwrap())).clone1(py).into_py_any(py).unwrap();
let mut blocks = (PyList::new(py, Vec::<Buffer>::new()).unwrap().unbind()).clone1(py).into_py_any(py).unwrap();
let mut i = (0).clone1(py).into_py_any(py).unwrap();
                    while (((i).clone1(py).into_py_any(py).unwrap()).extract::<usize>(py).unwrap()) < (((streamSize).clone1(py).into_py_any(py).unwrap()).extract::<usize>(py).unwrap()) {
                        let mut blockPtr = ((((streamPtr).clone1(py).into_py_any(py).unwrap()).extract::<Buffer>(py).unwrap()).borrow(py, (blocksize).clone1(py).into_py_any(py).unwrap(), (i).clone1(py).into_py_any(py).unwrap())).clone1(py).into_py_any(py).unwrap();
blocks.call_method(py, "append", ((blockPtr).clone1(py).into_py_any(py).unwrap(),), None).unwrap();
                        i = ((((i).clone1(py).into_py_any(py).unwrap()).extract::<usize>(py).unwrap()) + (((blocksize).clone1(py).into_py_any(py).unwrap()).extract::<usize>(py).unwrap())).clone1(py).into_py_any(py).unwrap();
                    }
                    
return (blocks).clone1(py).into_py_any(py).unwrap();
    return py.None();
}
        



// ====================

fn allocate(py: Python<'_>, size: Py<PyAny>) -> Buffer {
    let size = size.extract::<usize>(py).unwrap();
    Buffer::new(size)
}

fn free(py: Python<'_>, buffer: Py<PyAny>) {
    (buffer.extract::<Buffer>(py).unwrap()).free();
}

#[pyclass]
#[derive(Clone)]
struct Buffer {
    data: Arc<RwLock<Option<Vec<Byte>>>>,
}

impl Buffer {
    fn new(size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(Some(vec![Byte::new(0); size]))),
        }
    }

    fn free(&mut self) {
        let mut data = self
            .data
            .try_write()
            .expect("Can't free buffer while borrowed");

        let Some(data_vec) = data.as_mut() else {
            // already freed, do nothing
            return;
        };

        // check no bytes are borrowed
        for byte in data_vec.iter() {
            if byte.borrowed {
                panic!("Can't free buffer while borrowed");
            }
        }

        *data = None;
    }

    fn borrow(&self, py: Python<'_>, size: Py<PyAny>, index: Py<PyAny>) -> Py<PyList> {
        let size = size.extract::<usize>(py).unwrap();
        let index = index.extract::<usize>(py).unwrap();

        let mut data = self.data.write().unwrap();
        let data = data.as_mut().unwrap();

        let mut borrowed_data = vec![];
        for i in index..index + size {
            data[i].borrow();
            borrowed_data.push(data[i].data);
        }

        PyList::new(py, borrowed_data).unwrap().into()
    }

    fn borrowMut(&self, py: Python<'_>, size: Py<PyAny>, index: Py<PyAny>) -> Py<PyList> {
        let size = size.extract::<usize>(py).unwrap();
        let index = index.extract::<usize>(py).unwrap();

        let mut data = self.data.write().unwrap();
        let data = data.as_mut().unwrap();

        let mut borrowed_data = vec![];
        for i in index..index + size {
            if data[i].borrowed {
                panic!("Can't borrow mutably while borrowed");
            }

            data[i].borrow();
            borrowed_data.push(data[i].data);
        }

        PyList::new(py, borrowed_data).unwrap().into()
    }
}

#[pymethods]
impl Buffer {
    fn __getitem__(&self, index: isize) -> PyResult<u8> {
        let guard = self.data.read().unwrap();
        let data = guard.as_ref().unwrap();

        if index < 0 || index as usize >= data.len() {
            return Err(PyErr::new::<PyIndexError, _>("Index out of bounds"));
        }

        Ok(data[index as usize].data)
    }

    fn __setitem__(&mut self, index: isize, value: u8) -> PyResult<()> {
        let mut guard = self.data.write().unwrap();
        let data = guard.as_mut().unwrap();

        if index < 0 || index as usize >= data.len() {
            return Err(PyErr::new::<PyIndexError, _>("Index out of bounds"));
        }

        let byte = &mut data[index as usize];
        if byte.borrowed {
            return Err(PyErr::new::<PyIndexError, _>(
                "Can't mutate data while borrowed",
            ));
        }

        byte.data = value;
        Ok(())
    }

    fn __repr__(&self) -> PyResult<String> {
        let Ok(guard) = self.data.try_read() else {
            return Ok("Buffer(<mutably borrowed>)".to_string());
        };

        let data = guard.as_ref().unwrap();
        let hex = data
            .iter()
            .map(|byte| format!("{:02x}", byte.data))
            .collect::<Vec<String>>()
            .join(" ");

        Ok(format!("Buffer({hex})"))
    }
}

#[derive(Debug, Clone, Copy)]
struct Byte {
    data: u8,
    borrowed: bool,
}

impl Byte {
    fn new(data: u8) -> Self {
        Self {
            data,
            borrowed: false,
        }
    }

    fn borrow(&mut self) {
        self.borrowed = true;
    }

    fn release(&mut self) {
        self.borrowed = false;
    }
}

impl AsRef<u8> for Byte {
    fn as_ref(&self) -> &u8 {
        &self.data
    }
}

impl AsMut<u8> for Byte {
    fn as_mut(&mut self) -> &mut u8 {
        if self.borrowed {
            panic!("Can't mutate data while borrowed");
        }

        &mut self.data
    }
}

pub trait Clone1 {
    fn clone1(&self, py: Python<'_>) -> Self;
}

impl<T> Clone1 for Py<T> {
    fn clone1(&self, py: Python<'_>) -> Self {
        self.clone_ref(py)
    }
}

impl Clone1 for usize {
    fn clone1(&self, _: Python<'_>) -> Self {
        *self
    }
}
