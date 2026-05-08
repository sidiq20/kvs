mod store;

use store::KvStore;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // We'll store the log in a file called "kvs.log" in the current directory
    let path = PathBuf::from("kvs.log");
    let mut store = KvStore::open(path).expect("Failed to open store");

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "set" => {
            if args.len() != 4 {
                eprintln!("Error: 'set' requires 2 arguments");
                return;
            }
            store.set(args[2].clone(), args[3].clone()).expect("Failed to set");
            println!("OK");
        }
        "get" => {
            if args.len() != 3 {
                eprintln!("Error: 'get' requires 1 argument");
                return;
            }
            match store.get(args[2].clone()) {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        "rm" => {
            if args.len() != 3 {
                eprintln!("Error: 'rm' requires 1 argument");
                return;
            }
            store.remove(args[2].clone()).expect("Failed to remove");
            println!("OK");
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Usage: cargo run -- [set <k> <v> | get <k> | rm <k>]");
}
