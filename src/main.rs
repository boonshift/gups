use std::{env, fs, io, thread};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

use git2::Repository;

struct GitUpResult {
    repo_name: String,
    is_dirty: bool,
    branch: String,
    messages: String,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let base_dir = match &args.get(1) {
        Some(v) => v,
        None => "."
    };

    println!("Base Dir: {}", base_dir);

    let dirs = get_dirs(base_dir);
    let (tx, rx): (Sender<GitUpResult>, Receiver<GitUpResult>) = mpsc::channel();
    let mut children = Vec::new();

    for e in dirs {
//        println!("dir: {}", e.file_name().to_str().unwrap());
        let thread_tx = tx.clone();

        let child = thread::spawn(move || {
            let path_buf = e.path();
            explore_dir(path_buf, thread_tx);
        });

        children.push(child);
    }
    drop(tx);

    let printer = thread::spawn(move || {
        while let Ok(result) = rx.recv() {
            println!("Received from {}: {}", result.repo_name, result.messages);
        }
    });

    // Wait for the threads to complete any remaining work
    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    let _ = printer.join();

    Ok(())
}

fn get_dirs(base_dir: &str) -> Vec<DirEntry> {
    let dirs = fs::read_dir(base_dir).unwrap()
        .filter_map(Result::ok)
        .filter({
            |e| e.file_type().unwrap().is_dir()
        });

    return dirs.collect();
}

fn explore_dir(dir: PathBuf, tx: Sender<GitUpResult>) {
    let dir_path = dir.to_str().unwrap();
    let git_path = format!("{}/.git", dir_path);
    if !Path::new(&git_path).exists() {
        drop(tx);
        return;
    }

    let repo = match Repository::open(dir_path) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open: {}", e)
    };

    let head = repo.head().unwrap();
    let name = head.name().unwrap();
    let messages = String::new();

    let result = GitUpResult {
        repo_name: dir_path.to_string(),
        is_dirty: true,
        branch: name.to_string(),
        messages,
    };

    tx.send(result).unwrap();
    drop(tx);
}
