use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    ops::Deref,
    os::unix::prelude::FileExt,
    path::PathBuf,
};

use structopt::StructOpt;

#[derive(Debug, Default, StructOpt)]
enum Command {
    #[default]
    Attack,
    Restore,
}
#[derive(Debug, StructOpt)]
#[structopt(name = "cosmic-ray", about = "reverse bit")]
struct Opt {
    /// editing file path
    #[structopt(parse(from_os_str))]
    filepath: PathBuf,

    /// backup original file extension
    #[structopt(long, default_value = "orig")]
    origin: String,

    /// reverse byte address
    #[structopt(short, long)]
    pos: Option<u64>,

    /// reverse bit pattern
    #[structopt(long, default_value = "1")]
    pattern: u8,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

impl Opt {
    fn get_original_filepath(&self) -> PathBuf {
        let mut filepath = self.filepath.clone();
        filepath.set_extension(&self.origin);
        filepath
    }

    // 破壊されたファイルを消して、元のファイルを配置する
    fn restore(&self) -> std::io::Result<()> {
        let original_filepath = self.get_original_filepath();
        let Ok(_metadata) = fs::metadata(&original_filepath) else {
            return Ok(())
        };
        fs::remove_file(&self.filepath).err();
        fs::rename(&original_filepath, &self.filepath)?;
        log::info!("restore file {:?}", self.filepath);
        Ok(())
    }

    // 書き換えが可能かどうか
    fn setup(&mut self) -> std::io::Result<()> {
        use rand::Rng;

        // backup originalfile
        let original_filepath = self.get_original_filepath();
        let metadata = match fs::metadata(&original_filepath) {
            Err(_) => {
                let metadata = fs::metadata(&self.filepath)?;
                fs::copy(&self.filepath, &original_filepath)?;
                log::info!("backup original file {:?}", &original_filepath);
                metadata
            }
            Ok(metadata) => {
                if fs::metadata(&self.filepath).is_ok() {
                    fs::remove_file(&self.filepath)?
                };
                fs::copy(&original_filepath, &self.filepath)?;
                metadata
            }
        };

        // position更新 randam or overrange確認
        match self.pos {
            Some(pos) => {
                if pos > metadata.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("out of range pos={pos}, but file length={}", metadata.len()),
                    ));
                }
            }
            None => {
                let mut rng = rand::thread_rng();
                self.pos = Some(rng.gen_range(0..metadata.len()));
            }
        };
        Ok(())
    }

    fn do_reverse(&self) -> std::io::Result<()> {
        let mut buf = [0_u8];
        log::debug!("open {:?}", &self.filepath);
        let f = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&self.filepath)?;
        let offset = self.pos.unwrap();
        f.read_at(&mut buf, offset)?;
        let before = buf[0];
        log::debug!("read {} is {:#b}", offset, buf[0]);
        buf[0] ^= self.pattern;
        log::info!("write {} is {:#b} from {:#b}", offset, buf[0], before);
        f.write_at(&buf, offset)?;
        Ok(())
    }
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let mut opt = Opt::from_args();
    match opt.cmd {
        Command::Attack => {
            opt.setup().unwrap();
            log::debug!("success setup");
            opt.do_reverse().unwrap()
        }
        Command::Restore => opt.restore().unwrap(),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ray position is out of range")]
    OutOfRange,
    #[error("io error")]
    IO(#[from] std::io::Error),
}

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

pub fn affect(buf: &mut [u8], pat: &Ray) -> Result<(), Error> {
    let data = buf.get_mut(pat.offset).ok_or(Error::OutOfRange)?;
    *data ^= pat.pattern;
    Ok(())
}

pub struct RayBoxVec {
    data: Vec<u8>,
    rays: Vec<Ray>,
}

impl RayBoxVec {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, rays: vec![] }
    }

    pub fn attack(&mut self, ray: Ray) -> Result<(), Error> {
        affect(&mut self.data, &ray)?;
        self.rays.push(ray);
        Ok(())
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

pub struct RayBoxFile<T> {
    data: T,
    rays: Vec<Ray>,
}

impl<T> RayBoxFile<T>
where
    T: Read + Write + Seek,
{
    pub fn new(data: T) -> Self {
        Self { data, rays: vec![] }
    }

    #[inline]
    fn affect(&mut self, ray: &Ray) -> Result<(), Error> {
        let mut buf = [0_u8];
        let z_ray = ray.zero();
        self.data.seek(SeekFrom::Start(ray.offset as u64))?;
        let _pos = self.data.read(&mut buf)?;
        affect(&mut buf, &z_ray)?;
        self.data.seek(SeekFrom::Start(ray.offset as u64))?;
        let _pos = self.data.write(&buf)?;
        Ok(())
    }

    pub fn attack(&mut self, ray: Ray) -> Result<(), Error> {
        self.affect(&ray)?;
        self.rays.push(ray);
        Ok(())
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
        let Self { data, .. } = x;
        data
    }
}

impl<T> Deref for RayBoxFile<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, ops::Deref};

    use serde::{Deserialize, Serialize};
    use tempdir::TempDir;

    use crate::{affect, Error, Opt, Ray, RayBoxFile, RayBoxVec};

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
    fn test_file_reverse_restore() -> Result<(), std::io::Error> {
        let data = TestData {
            test1: "test1".to_string(),
            test2: vec![1, 2, 3, 4],
        };
        let orig_extension = "orig";

        let tmp_dir = TempDir::new("test")?;
        let target_file = tmp_dir.path().join("target.json");
        {
            let tmp_file = File::create(&target_file)?;
            serde_json::to_writer(tmp_file, &data)?;
        }

        let mut opt = Opt {
            filepath: target_file.clone(),
            origin: orig_extension.to_string(),
            pos: Some(0),
            pattern: 1,
            cmd: crate::Command::Attack,
        };

        opt.setup().unwrap();
        opt.do_reverse()?;

        // オリジナルは読めて、以前のなまrのファイルには破壊されたデータがある
        {
            let orig_file = tmp_dir.path().join(format!("target.{orig_extension}"));
            let f = File::open(orig_file)?;
            let orig_data: TestData = serde_json::from_reader(f).unwrap();
            assert_eq!(data, orig_data);
        }

        {
            let f = File::open(&target_file)?;
            let broken_data: Result<TestData, serde_json::Error> = serde_json::from_reader(f);
            match broken_data {
                Ok(_) => unreachable!(),
                Err(e) => {
                    println!("{}", e);
                }
            };
        }

        opt.restore()?;
        // オリジナルに戻っていること
        {
            let f = File::open(&target_file)?;
            let orig_data: TestData = serde_json::from_reader(f).unwrap();
            assert_eq!(data, orig_data);
        }

        Ok(())
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
