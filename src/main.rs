use std::{fs, os::unix::prelude::FileExt, path::PathBuf};

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

#[cfg(test)]
mod tests {
    use std::fs::File;

    use serde::{Deserialize, Serialize};
    use tempdir::TempDir;

    use crate::Opt;

    #[test]
    fn test_bit_reverse() {
        let pattern = 0b00000001_u8;
        assert_eq!(0b10000000 ^ pattern, 0b10000001);
        assert_eq!(0b11111111 ^ pattern, 0b11111110);
    }

    #[test]
    fn test_file_reverse_restore() -> Result<(), std::io::Error> {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            test1: String,
            test2: Vec<u8>,
        }

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
}
