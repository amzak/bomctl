use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

mod dir_walker;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    dry_run: bool,
}

fn try_read_bom(path: &PathBuf, buffer: &mut [u8]) -> std::io::Result<()>
{
    let _file = File::open(&path)?.read_exact(buffer)?;
    Ok(())
}

fn main()
{
    let options = Options::from_args();
    let current_dir = env::current_dir()
        .expect("can't get current directory");
    println!("checking boms in {:?} ...", current_dir);

    let files_list = dir_walker::DirWalker::new(current_dir)
        .expect(format!("can't enumerate files in {:?}", current_dir));

    let mut buffer: [u8; 3] = [0;3];
    let bom: [u8; 3] = [0xef, 0xbb, 0xbf];

    for path in files_list {
        if let Err(err) = try_read_bom(&path, &mut buffer) {
            println!("ERROR {:?}", err.to_string());
            continue;
        }
        
        print!("{:?} ", path);

        let has_bom = buffer.iter().zip(&bom).all(|(first, second)| first == second);

        if has_bom {
            println!("BOM");
        }
        else {
            println!("no BOM");
        }
    }
}