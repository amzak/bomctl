use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use lazy_static::lazy_static;
use std::collections::HashMap;

mod dir_walker;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    dry_run: bool,
    #[structopt(short, long)]
    verbose: bool
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Encodings {
    utf8,
    uft16le,
    uft16be
}

lazy_static! {
    static ref BOMS_MAP: HashMap<Encodings, Vec<u8>> = {
        let mut m = HashMap::new();
        m.insert(Encodings::utf8, vec![0xef, 0xbb, 0xbf]);
        m.insert(Encodings::uft16be, vec![0xfe, 0xff]);
        m.insert(Encodings::uft16le, vec![0xff, 0xfe]);
        m
    };
}

fn try_read_bom(path: &PathBuf, buffer: &mut [u8]) -> std::io::Result<bool> {
    let mut _file = File::open(&path)?;
    let meta = _file.metadata()?;
    let len = buffer.len() as u64;
    if meta.len() <= len {
        return Ok(false)
    }
    _file.read_exact(buffer)?;
    Ok(true)
}

fn main() {
    let options = Options::from_args();
    let current_dir = env::current_dir()
        .expect("can't get current directory");
    println!("checking in {:?} ...", current_dir);

    let files_list = dir_walker::DirWalker::new(&current_dir)
        .unwrap_or_else(|_| panic!("can't enumerate files in {:?}", &current_dir));

    let mut buffer: [u8; 3] = [0;3];
    let mut boms_counter = 0;

    for path in files_list {
        let bom_read_result = try_read_bom(&path, &mut buffer);

        match bom_read_result {
            Err(err) => {
                println!("ERROR {:?} {:?}", err.to_string(), path);
                continue;
            }
            Ok(result) => {
                if !result && options.verbose {
                    println!("ignored {:?}", path);
                    continue;
                }
            }
        }
                
        for (encoding, bom_signature) in BOMS_MAP.iter() {
            let has_bom = buffer.iter().zip(bom_signature).all(|(first, second)| first == second);

            if has_bom {
                println!("{:?} {:?} BOM", path, encoding);
                boms_counter += 1;
            }
        }
    }

    println!("{:?} files with BOM found", boms_counter);
}