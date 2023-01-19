use cosmic_ray;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyByteArray, PyBytes},
};

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

    fn raw<'p>(&'p self) -> &'p [u8] {
        &self.elements
    }

    fn reference<'p>(&'p self, py: Python<'p>) -> &'p PyBytes {
        PyBytes::new(py, &self.elements)
    }
}

#[pyfunction]
fn affect(buf: &PyByteArray) -> PyResult<()> {
    cosmic_ray::affect(unsafe { buf.as_bytes_mut() }, &cosmic_ray::Ray::new(0))
        .map_err(|e| PyValueError::new_err(format!("{}", e)))?;
    Ok(())
}

// moduleの実装定義
// 関数名はmoduke名にするか #[pyo3(name = "custom_name")] で指定する
#[pymodule]
#[pyo3(name = "cosmic_ray")]
fn extention(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Number>()?;
    m.add_class::<ByteBox>()?;
    m.add_function(wrap_pyfunction!(affect, m)?)?;
    Ok(())
}
