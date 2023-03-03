#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        if args.len() >= 2 {
            let p = std::path::Path::new(&args[2]);
            env::set_current_dir(p).unwrap();
        }

        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else {
        println!("unknown command: {}", args[1])
    }
}
