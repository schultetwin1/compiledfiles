extern crate compiledfiles;

use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: compilefiles <path>");
        process::exit(1);
    }

    let binary_path = Path::new(&args[1]);

    if !binary_path.exists() {
        println!("\"{}\" does not exist", binary_path.display());
        process::exit(1);
    }

    let file = match std::fs::File::open(binary_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening file \"{}\"", binary_path.display());
            print!("{}", e);
            process::exit(1);
        }
    };

    let files = match compiledfiles::parse(file) {
        Ok(files) => files,
        Err(err) => {
            match err {
                compiledfiles::Error::MissingDebugSymbols => {
                    println!("ERROR: \"{}\" missing debug symbols", binary_path.display(),);
                }
                _ => {
                    println!("ERROR: {}", err);
                }
            }
            process::exit(1);
        }
    };

    for file in files {
        println!("{:?}", file);
    }
}
