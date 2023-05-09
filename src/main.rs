use std::{env, fs, io, thread};
use std::fs::{DirEntry, FileType};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Instant;

use colored::Colorize;
use git2::{Repository, StatusOptions};

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
        let thread_tx = tx.clone();

        let child = thread::spawn(move || {
            let path_buf = e.path();
            explore_dir(path_buf, thread_tx);
        });

        children.push(child);
    }
    drop(tx);

    let printer = pass_recv_to_printer(rx);

    // Wait for the threads to complete any remaining work
    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    let _ = printer.join();

    Ok(())
}

fn pass_recv_to_printer(rx: Receiver<GitUpResult>) -> JoinHandle<()> {
    return thread::spawn(move || {
        while let Ok(r) = rx.recv() {
            if r.is_dirty {
                print!("{}", "**** ".red());
            }

            let branch = if r.branch != "refs/heads/master" { r.branch.as_str().cyan() } else { r.branch.as_str().green() };
            println!("Received from {} [{}]: {}", r.repo_name, branch, r.messages);
        }
    });
}

fn get_dirs(base_dir: &str) -> Vec<DirEntry> {
    let dirs = fs::read_dir(base_dir).unwrap()
        .filter_map(Result::ok)
        .filter({
            |e| -> bool {
                let file_type: FileType = e.file_type().unwrap();
                file_type.is_dir() || file_type.is_symlink()
            }
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

    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => panic!("Failed to get head for {}.", dir_path)
    };

    let name = head.name().unwrap();
    let mut messages = String::new();
    let mut is_dirty = true;
    if is_clean(&repo) {
        is_dirty = false;
        let start = Instant::now();

        let output = Command::new("zsh").current_dir(dir_path)
            .arg("-i").arg("-c")
            .arg("gfa >/dev/null 2>&1 && gup | grep '^Updating'; exit")
            .output()
            .expect("Failed to execute zsh.");
        let std_output = String::from_utf8(output.stdout).unwrap();

        let elapsed = start.elapsed();
        let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        messages.push_str(format!("Updated in {} secs: {}", secs, std_output.trim().yellow()).as_str());
    }

    let result = GitUpResult {
        repo_name: dir_path.to_string(),
        is_dirty,
        branch: name.to_string(),
        messages,
    };


    tx.send(result).unwrap();
    drop(tx);
}

fn is_clean(repo: &Repository) -> bool {
    let mut status_options: StatusOptions = StatusOptions::new();
    let statuses = repo.statuses(Some(&mut status_options)).unwrap();
    return statuses.is_empty();
}
