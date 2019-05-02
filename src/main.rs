use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let base_dir = match &args.get(1) {
        Some(v) => v,
        None => "."
    };

    println!("{}", base_dir);
}
