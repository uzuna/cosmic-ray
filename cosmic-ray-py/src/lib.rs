use cosmic_ray;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyByteArray};
use rand::Rng;

// #[pyclass]
// struct Number(i32);

// #[pymethods]
// impl Number {
//     #[new]
//     fn py_new(value: i32) -> PyResult<Self> {
//         if value == 0 {
//             Err(PyValueError::new_err("cannot be zero"))
//         } else {
//             Ok(Number(value))
//         }
//     }
//     fn add(&mut self, rhs: i32) -> PyResult<i32> {
//         self.0 += rhs;
//         Ok(self.0)
//     }
//     fn get(&self) -> PyResult<i32> {
//         Ok(self.0)
//     }
// }

// #[pyclass]
// struct ByteBox {
//     #[pyo3(get)]
//     elements: Vec<u8>,
// }

// #[pymethods]
// impl ByteBox {
//     #[new]
//     fn py_new(value: &PyBytes) -> PyResult<Self> {
//         let elements = value.as_bytes().to_vec();
//         Ok(Self { elements })
//     }

//     fn raw<'p>(&'p self) -> &'p [u8] {
//         &self.elements
//     }

//     fn reference<'p>(&'p self, py: Python<'p>) -> &'p PyBytes {
//         PyBytes::new(py, &self.elements)
//     }
// }

#[pyclass]
#[repr(transparent)]
struct Ray(cosmic_ray::Ray);

#[pymethods]
impl Ray {
    #[new]
    fn py_new(offset: usize) -> Self {
        Ray(cosmic_ray::Ray::new(offset))
    }
}

#[pyclass]
struct RayBox {
    history: Vec<cosmic_ray::Ray>,
}

#[pymethods]
impl RayBox {
    #[new]
    fn py_new() -> Self {
        Self { history: vec![] }
    }

    fn attack(&mut self, buf: &PyByteArray, ray: &Ray) -> PyResult<()> {
        affect(buf, &ray.0)?;
        self.history.push(ray.0.clone());
        Ok(())
    }

    fn attack_rand(&mut self, buf: &PyByteArray) -> PyResult<()> {
        let max = unsafe { buf.as_bytes().len() };
        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(0..max);
        let ray = cosmic_ray::Ray::new(offset);
        affect(buf, &ray)?;
        self.history.push(ray);
        Ok(())
    }

    fn restore(&mut self, buf: &PyByteArray) -> PyResult<()> {
        for ray in self.history.iter().rev() {
            affect(buf, ray)?;
        }
        self.history.clear();
        Ok(())
    }
}

fn affect(buf: &PyByteArray, ray: &cosmic_ray::Ray) -> PyResult<()> {
    cosmic_ray::affect(unsafe { buf.as_bytes_mut() }, ray)
        .map_err(|e| PyValueError::new_err(format!("{}", e)))?;
    Ok(())
}

// moduleの実装定義
// 関数名はmodule名にするか #[pyo3(name = "custom_name")] で指定する
#[pymodule]
#[pyo3(name = "cosmic_ray")]
fn extention(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<RayBox>()?;
    m.add_class::<Ray>()?;
    Ok(())
}
