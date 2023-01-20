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

/// Information about which bit at which position to invert
#[pyclass]
#[repr(transparent)]
struct Ray(cosmic_ray::Ray);

#[pymethods]
impl Ray {
    /// アドレス指定して1ビット反転させる
    #[new]
    fn py_new(offset: usize) -> Self {
        Ray(cosmic_ray::Ray::new(offset))
    }

    /// アドレスとビットパターンを指定してビット反転させる
    #[staticmethod]
    fn with_pattern(offset: usize, pattern: u8) -> Self {
        Ray(cosmic_ray::Ray::with_pattern(offset, pattern))
    }
}

/// バッファの操作記録を保持する構造体
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

    /// 指定したパターンでビット反転させる
    fn attack(&mut self, buf: &PyByteArray, ray: &Ray) -> PyResult<()> {
        affect(buf, &ray.0)?;
        self.history.push(ray.0.clone());
        Ok(())
    }

    /// ランダムなアドレスを反転させる
    fn attack_random(&mut self, buf: &PyByteArray) -> PyResult<()> {
        let max = unsafe { buf.as_bytes().len() };
        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(0..max);
        let ray = cosmic_ray::Ray::new(offset);
        affect(buf, &ray)?;
        self.history.push(ray);
        Ok(())
    }

    /// ランダムなアドレスのどこかのビットを反転させる
    fn attack_randam_with_pattern(&mut self, buf: &PyByteArray) -> PyResult<()> {
        let max = unsafe { buf.as_bytes().len() };
        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(0..max);
        let pattern = 2_u8.pow(rng.gen_range(0..=8));
        let ray = cosmic_ray::Ray::with_pattern(offset, pattern);
        affect(buf, &ray)?;
        self.history.push(ray);
        Ok(())
    }

    /// 元のバイト列に戻す
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
#[pyo3(name = "cosmic_ray_py")]
fn extention(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<RayBox>()?;
    m.add_class::<Ray>()?;
    Ok(())
}
