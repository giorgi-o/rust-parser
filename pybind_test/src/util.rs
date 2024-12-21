use pyo3::{
    exceptions::PyIndexError, prelude::*, types::PyList, types::PyListMethods, IntoPyObjectExt,
};
use std::sync::{Arc, RwLock};

pub fn allocate(py: Python<'_>, size: &impl Var) -> Buffer {
    let size = size.to_usize(py);
    Buffer::new(size)
}

pub fn free(py: Python<'_>, buffer: &impl Var) {
    let mut buffer = buffer.to_buffer(py);
    buffer.free();
}

pub fn blackbox<'a, V: Var>(_py: Python<'_>, v: &'a V) -> &'a V {
    std::hint::black_box(v)
}

#[pyclass]
#[derive(Clone)]
pub struct Buffer {
    data: Arc<RwLock<Option<Vec<Byte>>>>,
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(Some(vec![Byte::new(0); size]))),
        }
    }

    pub fn free(&mut self) {
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

    pub fn borrow(&self, py: Python<'_>, size: &impl Var, index: &impl Var) -> Py<PyList> {
        let size = size.to_usize(py);
        let index = index.to_usize(py);

        let mut data = self.data.write().unwrap();
        let data = data.as_mut().unwrap();

        let mut borrowed_data = vec![];
        for i in index..index + size {
            data[i].borrow();
            borrowed_data.push(data[i].data);
        }

        PyList::new(py, borrowed_data).unwrap().into()
    }

    pub fn borrowMut(&self, py: Python<'_>, size: &impl Var, index: &impl Var) -> Py<PyList> {
        let size = size.to_usize(py);
        let index = index.to_usize(py);

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
    pub fn __getitem__(&self, index: isize) -> PyResult<u8> {
        let guard = self.data.read().unwrap();
        let data = guard.as_ref().unwrap();

        if index < 0 || index as usize >= data.len() {
            return Err(PyErr::new::<PyIndexError, _>("Index out of bounds"));
        }

        Ok(data[index as usize].data)
    }

    pub fn __setitem__(&mut self, index: isize, value: u8) -> PyResult<()> {
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

    pub fn __repr__(&self) -> PyResult<String> {
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
    pub fn new(data: u8) -> Self {
        Self {
            data,
            borrowed: false,
        }
    }

    pub fn borrow(&mut self) {
        self.borrowed = true;
    }

    pub fn release(&mut self) {
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

pub trait Var {
    fn to_pyany(&self, py: Python<'_>) -> Py<PyAny>;
    fn to_pylist<T>(&self, py: Python<'_>) -> Py<PyList>;
    fn to_buffer(&self, py: Python<'_>) -> Buffer;
    fn to_usize(&self, py: Python<'_>) -> usize;
}

impl Var for Py<PyAny> {
    fn to_pyany(&self, py: Python<'_>) -> Py<PyAny> {
        self.clone_ref(py)
    }

    fn to_pylist<T>(&self, py: Python<'_>) -> Py<PyList> {
        let list: &Bound<'_, PyList> = self.downcast_bound(py).unwrap();
        list.clone().unbind()
    }

    fn to_buffer(&self, py: Python<'_>) -> Buffer {
        self.extract::<Buffer>(py).unwrap()
    }

    fn to_usize(&self, py: Python<'_>) -> usize {
        self.extract::<usize>(py).unwrap()
    }
}

impl Var for Py<PyList> {
    fn to_pyany(&self, py: Python<'_>) -> Py<PyAny> {
        self.as_any().clone_ref(py)
    }

    fn to_pylist<T>(&self, py: Python<'_>) -> Py<PyList> {
        self.clone_ref(py)
    }

    fn to_buffer(&self, py: Python<'_>) -> Buffer {
        let mut data = Vec::<Byte>::new();
        self.bind(py).iter().for_each(|byte| {
            // try extracting usize, byte, and buffer
            if let Ok(byte) = byte.extract::<u8>() {
                data.push(Byte::new(byte));
            } else if let Ok(buffer) = byte.extract::<Buffer>() {
                let guard = buffer.data.read().unwrap();
                let buffer_data = guard.as_ref().unwrap();
                data.extend_from_slice(buffer_data);
            } else {
                panic!("Can't convert PyList to Buffer");
            }
        });

        Buffer {
            data: Arc::new(RwLock::new(Some(data))),
        }
    }

    fn to_usize(&self, py: Python<'_>) -> usize {
        self.extract::<usize>(py).unwrap()
    }
}

impl Var for Buffer {
    fn to_pyany(&self, py: Python<'_>) -> Py<PyAny> {
        self.clone().into_py_any(py).unwrap()
    }

    fn to_pylist<T>(&self, py: Python<'_>) -> Py<PyList> {
        let guard = self.data.read();
        let data = guard.as_ref().unwrap().as_ref().unwrap();
        PyList::new(py, data.iter().map(|byte| byte.data).collect::<Vec<u8>>())
            .unwrap()
            .unbind()
    }

    fn to_buffer(&self, _: Python<'_>) -> Buffer {
        self.clone()
    }

    fn to_usize(&self, _: Python<'_>) -> usize {
        self.data.read().unwrap().as_ref().unwrap().len()
    }
}

impl Var for usize {
    fn to_pyany(&self, py: Python<'_>) -> Py<PyAny> {
        (*self).into_py_any(py).unwrap()
    }

    fn to_pylist<T>(&self, _py: Python<'_>) -> Py<PyList> {
        panic!("Can't convert usize to PyList")
    }

    fn to_buffer(&self, _py: Python<'_>) -> Buffer {
        panic!("Can't convert usize to Buffer")
    }

    fn to_usize(&self, _: Python<'_>) -> usize {
        *self
    }
}
