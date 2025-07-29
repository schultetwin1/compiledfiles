extern crate compiledfiles;

use std::env;
use std::path::Path;
use std::process;

fn print_usage() {
    println!("usage: compilefiles [--json] <path>");
    println!("  --json    Output results in JSON format");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (json_output, binary_path) = if args.len() == 2 {
        (false, &args[1])
    } else if args.len() == 3 && args[1] == "--json" {
        (true, &args[2])
    } else {
        print_usage();
        process::exit(1);
    };

    let binary_path = Path::new(binary_path);

    if !binary_path.exists() {
        println!("\"{}\" does not exist", binary_path.display());
        process::exit(1);
    }

    let file = match std::fs::File::open(binary_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening file \"{}\"", binary_path.display());
            print!("{e}");
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
                    println!("ERROR: {err}");
                }
            }
            process::exit(1);
        }
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&files).expect("Failed to serialize files to JSON"));
    } else {
        for file in files {
            println!("{file:?}");
        }
    }
}
