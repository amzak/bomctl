use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Read, SeekFrom};
use std::path::{PathBuf, Path};
use structopt::StructOpt;
use lazy_static::lazy_static;
use std::collections::HashMap;

mod dir_walker;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    dry_run: bool,
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(parse(from_os_str))]
    working_dir: Option<PathBuf>
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Encodings {
    Utf8,
    Uft16le,
    Uft16be
}

struct FileResult {
    path: PathBuf,
    has_bom: bool,
    error: Option<String>
}

lazy_static! {
    static ref BOMS_MAP: HashMap<Encodings, Vec<u8>> = {
        let mut m = HashMap::new();
        m.insert(Encodings::Utf8, vec![0xef, 0xbb, 0xbf]);
        m.insert(Encodings::Uft16be, vec![0xfe, 0xff]);
        m.insert(Encodings::Uft16le, vec![0xff, 0xfe]);
        m
    };
}

fn try_read_bom(path: &Path, buffer: &mut [u8]) -> std::io::Result<bool> {
    let mut _file = File::open(path)?;
    let meta = _file.metadata()?;
    let len = buffer.len() as u64;
    if meta.len() <= len {
        return Ok(false)
    }
    _file.read_exact(buffer)?;

    Ok(true)
}

fn trim_file_start(path: &Path, len: u64) -> std::io::Result<()> {
    let mut backup_path = PathBuf::from(path);
    backup_path.set_extension("bak");
    fs::rename(path, &backup_path)?;
    let mut file_in = File::open(backup_path)?;

    let file_len = file_in.metadata()?.len();
    let mut buffer = Vec::with_capacity(file_len as usize);

    file_in.seek(SeekFrom::Start(len))?;
    file_in.read_to_end(&mut buffer)?;

    let mut file_out = File::create(path)?;
    file_out.write_all(&buffer)?;

    return Ok(())
}

fn output_formatted(results: &[FileResult]) {
    for result in results {
        let result_error = &(result.error); 
        if result_error.is_none() {
            if result.has_bom {
                println!("{:?} BOM", result.path);
            }
            else {
                println!("{:?} no BOM", result.path);
            }
            continue;
        }
        if let Some(error) = result_error {
            println!("ERROR {:?} {:?}", error, result.path);
        }
    }
}

fn main() {
    let ref mut options = Options::from_args();

    if options.working_dir.is_none() {
        options.working_dir = Some(env::current_dir().expect("can't get current directory"));
    }

    println!("checking in {:?} ...", options.working_dir);

    match _main(options) {
        Err(error) => panic!("error {:?} during files processing in {:?}", 
            error.to_string(), 
            options.working_dir),
        Ok(results) => output_formatted(&results)
    }
}

fn _main(options: &Options) -> std::io::Result<Vec<FileResult>> {
    let current_dir = options.working_dir.as_ref().unwrap();

    let mut results = Vec::new();

    let files_list = dir_walker::DirWalker::new(&current_dir)?;

    let mut buffer: [u8; 3] = [0;3];

    for path in files_list {
        let bom_read_result = try_read_bom(&path, &mut buffer);

        match bom_read_result {
            Err(err) => {
                let error = err.to_string();
                results.push(FileResult{path: path.clone(), has_bom: false, error: Some(error)});
                continue;
            }
            Ok(result) => {
                if !result && options.verbose {
                    results.push(FileResult{path: path.clone(), has_bom: false, error: None});
                    continue;
                }
            }
        }
                
        for (_encoding, bom_signature) in BOMS_MAP.iter() {
            let has_bom = buffer.iter().zip(bom_signature).all(|(first, second)| first == second);

            if !has_bom {
                continue;
            }

            results.push(FileResult{path: path.clone(), has_bom: true, error: None});

            if options.dry_run {
                continue;
            }

            if let Err(error) = trim_file_start(&path, bom_signature.len() as u64) {
                results.push(FileResult{path: path.clone(), has_bom: true, error: Some(error.to_string())});
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod bomctl_tests {
    use super::*;

    const TESTS_IN_DIR: &str = "target/test";
    const TESTS_FILE: &str = "test_file";

    fn file_full_path() -> PathBuf {
        Path::new(TESTS_IN_DIR).join(TESTS_FILE)
    }

    fn backup_file_full_path() -> PathBuf {
        let mut path = file_full_path();
        path.set_extension("bak");
        path
    }
    
    fn gen_target() {
        fs::create_dir_all(TESTS_IN_DIR).unwrap();
        let mut file = File::create(file_full_path()).unwrap();
        let bom = [0xef, 0xbb, 0xbf];
        file.write_all(&bom).unwrap();
        file.write_all(b"Hello, world!").unwrap();
    }

    fn setup() {
        gen_target();
    }

    fn cleanup() {
        fs::remove_file(file_full_path()).unwrap();

        if backup_file_full_path().exists() {
            fs::remove_file(backup_file_full_path()).unwrap();
        }

        fs::remove_dir(Path::new(TESTS_IN_DIR)).unwrap();
    }

    #[test]
    fn should() {
        setup();

        let options = Options { 
            dry_run: false, 
            verbose: true,
            working_dir: Some(PathBuf::from(TESTS_IN_DIR))
        };
        let results = _main(&options);

        let target_processed = results.unwrap()
            .iter()
            .any(|item| item.path == file_full_path());

        assert_eq!(target_processed, true);

        cleanup();
    }
}
