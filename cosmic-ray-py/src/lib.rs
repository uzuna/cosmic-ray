use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};

#[pyclass]
struct Number(i32);

#[pymethods]
impl Number {
    #[new]
    fn py_new(value: i32) -> PyResult<Self> {
        if value == 0 {
            Err(PyValueError::new_err("cannot be zero"))
        } else {
            Ok(Number(value))
        }
    }
    fn add(&mut self, rhs: i32) -> PyResult<i32> {
        self.0 += rhs;
        Ok(self.0)
    }
    fn get(&self) -> PyResult<i32> {
        Ok(self.0)
    }
}

#[pyclass]
struct ByteBox {
    #[pyo3(get)]
    elements: Vec<u8>,
}

#[pymethods]
impl ByteBox {
    #[new]
    fn py_new(value: &PyBytes) -> PyResult<Self> {
        let elements = value.as_bytes().to_vec();
        Ok(Self { elements })
    }
}

// impl FromPyObject<'_> for ByteBox {
//     fn extract(ob: &PyAny) -> PyResult<Self> {
//         let byte: &PyBytes = ob.downcast()?;
//         let elements = byte.as_bytes().to_vec();
//         Ok(Self{elements})
//     }
// }

// #[pyproto]
// impl PySequenceProtocol for ByteBox {
//     fn __concat__(&self, other: PyRef<'p, Self>) -> PyResult<Self> {
//         let mut elements = self.elements.clone();
//         elements.extend_from_slice(&other.elements);
//         Ok(Self { elements })
//     }
// }

// moduleの実装定義
// 関数名はmoduke名にするか #[pyo3(name = "custom_name")] で指定する
#[pymodule]
fn cosmic_ray(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Number>()?;
    m.add_class::<ByteBox>()?;
    Ok(())
}
