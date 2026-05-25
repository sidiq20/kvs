mod store;

use store::{KvStore, Command, Response};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "server" => run_server(),
        "set" | "get" | "rm" => run_client(args),
        _ => print_usage(),
    }
}

// --- SERVER CODE ---
fn run_server() {
    let path = PathBuf::from("kvs.log");
    let mut store = KvStore::open(path).expect("Failed to open store");
    
    let listener = TcpListener::bind("127.0.0.1:4000").expect("Could not bind to port 4000");
    println!("Server listening on 127.0.0.1:4000...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Read the command from the client
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                
                if reader.read_line(&mut line).is_ok() {
                    if let Ok(cmd) = serde_json::from_str::<Command>(&line) {
                        
                        // Execute the command
                        let response = match cmd {
                            Command::Set { key, value } => {
                                match store.set(key, value) {
                                    Ok(_) => Response::Ok(None),
                                    Err(e) => Response::Err(e.to_string()),
                                }
                            }
                            Command::Get { key } => {
                                Response::Ok(store.get(key))
                            }
                            Command::Remove { key } => {
                                match store.remove(key) {
                                    Ok(_) => Response::Ok(None),
                                    Err(e) => Response::Err(e.to_string()),
                                }
                            }
                        };
                        
                        // Send the response back
                        let res_json = serde_json::to_string(&response).unwrap();
                        writeln!(stream, "{}", res_json).unwrap();
                    }
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

// --- CLIENT CODE ---
fn run_client(args: Vec<String>) {
    let cmd = match args[1].as_str() {
        "set" => {
            if args.len() != 4 { return eprintln!("Error: 'set' requires 2 arguments"); }
            Command::Set { key: args[2].clone(), value: args[3].clone() }
        }
        "get" => {
            if args.len() != 3 { return eprintln!("Error: 'get' requires 1 argument"); }
            Command::Get { key: args[2].clone() }
        }
        "rm" => {
            if args.len() != 3 { return eprintln!("Error: 'rm' requires 1 argument"); }
            Command::Remove { key: args[2].clone() }
        }
        _ => return,
    };

    // Connect to the server
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:4000") {
        // Send command
        let cmd_json = serde_json::to_string(&cmd).unwrap();
        writeln!(stream, "{}", cmd_json).unwrap();

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        if reader.read_line(&mut response_line).is_ok() {
            if let Ok(response) = serde_json::from_str::<Response>(&response_line) {
                match response {
                    Response::Ok(Some(val)) => println!("{}", val),
                    Response::Ok(None) => println!("OK"),
                    Response::Err(err) => eprintln!("Server Error: {}", err),
                }
            }
        }
    } else {
        eprintln!("Failed to connect to server. Is it running?");
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run -- server");
    println!("  cargo run -- set <k> <v>");
    println!("  cargo run -- get <k>");
    println!("  cargo run -- rm <k>");
}