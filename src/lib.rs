//! Cosmic Ray is a Flip Bit emulator.
//!
//! Unfortunately, unintended bit flipping can occur in computer memory.
//! It's very rare, but it can happen if you use it for a long time.
//! This tool intentionally flips the bit and is there
//! to test how robust your tool is to breaking it.
//!
//! # Example
//!
//! ```rust
//! use cosmic_ray::{RayBoxVec, Ray};
//!
//! let buf = vec![0_u8; 12];
//! let reference = buf.clone();
//!
//! let mut raybox = RayBoxVec::new(buf);
//! raybox.attack(Ray::new(0)).unwrap();
//! assert_ne!(&*raybox, &reference);
//! raybox.restore();
//! assert_eq!(&*raybox, &reference);
//! ```
use std::{
    io::{Read, Seek, SeekFrom, Write},
    ops::Deref,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ray position is out of range")]
    OutOfRange,
    #[error("io error")]
    IO(#[from] std::io::Error),
}

/// Information about which bit at which position to invert
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub offset: usize,
    pub pattern: u8,
}

impl Ray {
    pub const P0BIT: u8 = 0b00000001;
    pub const P1BIT: u8 = 0b00000010;
    pub const P2BIT: u8 = 0b00000100;
    pub const P3BIT: u8 = 0b00001000;
    pub const P4BIT: u8 = 0b00010000;
    pub const P5BIT: u8 = 0b00100000;
    pub const P6BIT: u8 = 0b01000000;
    pub const P7BIT: u8 = 0b10000000;

    pub fn new(offset: usize) -> Self {
        Self {
            offset,
            pattern: Self::P0BIT,
        }
    }

    pub fn with_pattern(offset: usize, pattern: u8) -> Self {
        Self { offset, pattern }
    }

    fn zero(&self) -> Self {
        Self {
            offset: 0,
            pattern: self.pattern,
        }
    }
}

/// Rewrites bytes according to Ray information
/// Since it is an XOR operation, it will return to the original data after applying it twice.
pub fn affect(buf: &mut [u8], pat: &Ray) -> Result<u8, Error> {
    let data = buf.get_mut(pat.offset).ok_or(Error::OutOfRange)?;
    *data ^= pat.pattern;
    Ok(*data)
}

/// It keeps a record of bytes and destructive operations.
/// Safely perform multiple destructions and restores to the original data.
pub struct RayBoxVec {
    data: Vec<u8>,
    rays: Vec<Ray>,
}

impl RayBoxVec {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, rays: vec![] }
    }

    pub fn attack(&mut self, ray: Ray) -> Result<u8, Error> {
        let result = affect(&mut self.data, &ray)?;
        self.rays.push(ray);
        Ok(result)
    }

    pub fn restore(&mut self) -> Option<Ray> {
        if let Some(ray) = self.rays.pop() {
            // attackが通ったなら通常は確実に通るのでチェックしない
            unsafe {
                affect(&mut self.data, &ray).unwrap_unchecked();
            }
            Some(ray)
        } else {
            None
        }
    }

    pub fn restore_all(&mut self) {
        for ray in self.rays.iter().rev() {
            unsafe {
                affect(&mut self.data, ray).unwrap_unchecked();
            }
        }
        self.rays.clear();
    }

    #[inline]
    pub fn is_damaged(&self) -> bool {
        !self.rays.is_empty()
    }

    pub fn into_inner(x: Self) -> Vec<u8> {
        let Self { data, .. } = x;
        data
    }
}

// バイト列比較のため
impl Deref for RayBoxVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// It keeps a ByteIO and destructive operations.
/// Safely perform multiple destructions and restores to the original data.
pub struct RayBoxFile<T> {
    file: T,
    rays: Vec<Ray>,
}

impl<T> RayBoxFile<T>
where
    T: Read + Write + Seek,
{
    pub fn new(file: T) -> Self {
        Self { file, rays: vec![] }
    }

    #[inline]
    fn affect(&mut self, ray: &Ray) -> Result<u8, Error> {
        let mut buf = [0_u8];
        let z_ray = ray.zero();
        self.file.seek(SeekFrom::Start(ray.offset as u64))?;
        let _pos = self.file.read(&mut buf)?;
        let result = affect(&mut buf, &z_ray)?;
        self.file.seek(SeekFrom::Start(ray.offset as u64))?;
        let _pos = self.file.write(&buf)?;
        Ok(result)
    }

    pub fn attack(&mut self, ray: Ray) -> Result<u8, Error> {
        let result = self.affect(&ray)?;
        self.rays.push(ray);
        Ok(result)
    }

    pub fn restore(&mut self) -> Option<Ray> {
        if let Some(ray) = self.rays.pop() {
            // attackが通ったなら通常は確実に通るのでチェックしない
            self.affect(&ray).unwrap();
            Some(ray)
        } else {
            None
        }
    }

    pub fn restore_all(&mut self) {
        let rays: Vec<Ray> = self.rays.drain(0..).collect();
        for ray in rays.iter().rev() {
            self.affect(ray).unwrap();
        }
    }

    #[inline]
    pub fn is_damaged(&self) -> bool {
        !self.rays.is_empty()
    }
    pub fn into_inner(x: Self) -> T {
        let Self { file: data, .. } = x;
        data
    }
}

impl<T> Deref for RayBoxFile<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, ops::Deref};

    use serde::{Deserialize, Serialize};
    use tempdir::TempDir;

    use crate::{affect, Error, Ray, RayBoxFile, RayBoxVec};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        test1: String,
        test2: Vec<u8>,
    }

    #[test]
    fn test_bit_reverse() {
        let pattern = 0b00000001_u8;
        assert_eq!(0b10000000 ^ pattern, 0b10000001);
        assert_eq!(0b11111111 ^ pattern, 0b11111110);
    }

    #[test]
    fn test_bytes() -> Result<(), Error> {
        let mut buf = vec![0_u8; 12];
        let reference = buf.clone();

        let ray = Ray::new(0);
        affect(&mut buf, &ray)?;
        assert_ne!(&buf, &reference);
        affect(&mut buf, &ray)?;
        assert_eq!(&buf, &reference);
        Ok(())
    }

    #[test]
    fn test_raybox() -> Result<(), Error> {
        let buf = vec![0_u8; 12];
        let reference = buf.clone();

        let mut raybox = RayBoxVec::new(buf);
        assert_eq!(&*raybox, &reference);

        for ray in [
            Ray::with_pattern(0, Ray::P0BIT),
            Ray::with_pattern(2, Ray::P1BIT),
            Ray::with_pattern(4, Ray::P2BIT),
            Ray::with_pattern(5, Ray::P3BIT),
            Ray::with_pattern(6, Ray::P4BIT),
            Ray::with_pattern(7, Ray::P5BIT),
            Ray::with_pattern(10, Ray::P6BIT),
            Ray::with_pattern(11, Ray::P7BIT),
        ] {
            raybox.attack(ray)?;
        }
        while raybox.is_damaged() {
            assert_ne!(&*raybox, &reference);
            raybox.restore();
        }
        assert_eq!(&*raybox, &reference);
        let buf = RayBoxVec::into_inner(raybox);
        assert_eq!(&buf, &reference);
        Ok(())
    }

    #[test]
    fn test_rayboxfile() -> Result<(), Error> {
        let data = TestData {
            test1: "test1".to_string(),
            test2: vec![1, 2, 3, 4],
        };
        let tmp_dir = TempDir::new("test")?;
        let target_file = tmp_dir.path().join("target.json");
        let reference_buf = {
            let tmp_file = File::create(&target_file)?;
            serde_json::to_writer(tmp_file, &data).unwrap();
            serde_json::to_vec(&data).unwrap()
        };

        let f = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&target_file)?;
        let mut raybox = RayBoxFile::new(f);

        for ray in [
            Ray::with_pattern(0, Ray::P0BIT),
            Ray::with_pattern(2, Ray::P1BIT),
            Ray::with_pattern(4, Ray::P2BIT),
            Ray::with_pattern(5, Ray::P3BIT),
            Ray::with_pattern(6, Ray::P4BIT),
            Ray::with_pattern(7, Ray::P5BIT),
            Ray::with_pattern(10, Ray::P6BIT),
            Ray::with_pattern(11, Ray::P7BIT),
        ] {
            raybox.attack(ray)?;
        }
        raybox.deref().flush().unwrap();
        let edited_byte = std::fs::read(&target_file)?;
        assert_ne!(&*edited_byte, &reference_buf);
        raybox.restore_all();
        drop(raybox);
        let restored_byte = std::fs::read(&target_file)?;
        assert_eq!(&restored_byte, &reference_buf);
        Ok(())
    }
}
