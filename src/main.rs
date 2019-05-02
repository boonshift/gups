use std::{env, fs, io};
use std::fs::DirEntry;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let base_dir = match &args.get(1) {
        Some(v) => v,
        None => "."
    };

    println!("Base Dir: {}", base_dir);

    let dirs = fs::read_dir(base_dir).unwrap()
        .filter_map(Result::ok)
        .filter({
            |e| e.file_type().unwrap().is_dir()
        });

    dirs.for_each(|e: DirEntry|
        println!("dir: {}", e.file_name().to_str().unwrap()));

    Ok(())
}
