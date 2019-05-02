use std::{env, fs, io, thread};
use std::fs::DirEntry;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let base_dir = match &args.get(1) {
        Some(v) => v,
        None => "."
    };

    println!("Base Dir: {}", base_dir);

    let dirs = get_dirs(base_dir);
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut children = Vec::new();

    for e in dirs {
//        println!("dir: {}", e.file_name().to_str().unwrap());
        let thread_tx = tx.clone();

        let child = thread::spawn(move || {
            let dir_name_string = e.file_name();
            let dir_name = dir_name_string.to_str().clone().unwrap();

            // The thread takes ownership over `thread_tx`
            // Each thread queues a message in the channel
            let msg = format!("hello from {}", dir_name);
            thread_tx.send(msg).unwrap();

            // Sending is a non-blocking operation, the thread will continue
            // immediately after sending its message
//            println!("thread {} finished", dir_name);
        });

        children.push(child);
    }

    let printer = thread::spawn(move || {
        loop {
            let msg = rx.recv().unwrap();
            println!("Received: {}", msg);
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
