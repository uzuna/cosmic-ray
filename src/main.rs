use std::{
    fs::{self, File},
    os::unix::prelude::FileExt,
    path::PathBuf,
};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "cosmic-ray", about = "reverse bit")]
struct Opt {
    /// editing file path
    #[structopt(parse(from_os_str))]
    filepath: PathBuf,

    /// backup original file extension
    #[structopt(long, default_value = ".orig")]
    origin: String,

    /// reverse byte address
    #[structopt(short, long)]
    pos: Option<u64>,

    /// reverse bit pattern
    #[structopt(long, default_value = "1")]
    pattern: u8,
}

impl Opt {
    fn get_original_filepath(&self) -> PathBuf {
        self.filepath.join(&self.origin)
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
                if metadata.len() > pos {
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
        let f = File::open(&self.filepath)?;
        let offset = self.pos.unwrap();
        f.read_at(&mut buf, offset)?;
        buf[0] ^= self.pattern;
        f.write_at(&buf, offset)?;
        Ok(())
    }
}

fn main() {
    let mut opt = Opt::from_args();
    opt.setup().unwrap();
    opt.do_reverse().unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bit_reverse() {
        let pattern = 0b00000001_u8;
        assert_eq!(0b10000000 ^ pattern, 0b10000001);
        assert_eq!(0b11111111 ^ pattern, 0b11111110);
    }
}
