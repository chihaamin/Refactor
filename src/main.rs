use std::env;
use std::path::PathBuf;
use std::process;
mod imports_resolver;
mod package_resolver;

fn main() {
    // Getting the working directory
    let current_dir: PathBuf = env::current_dir().unwrap_or_else(|err| {
        eprintln!("Error getting current directory: {}", err);
        process::exit(1);
    });
    // Getting cmd arguments
    let args: Vec<String> = env::args().collect();

    // Expecting a JS library
    if args.len() < 2 {
        eprintln!("Usage: refactor <some-js-libs>");
        process::exit(0); // Exit gracefully
    }

    // Extract the <some-js-lib> argument
    // todo multiple libs refactor
    let js_lib = &args[1];

    let export = package_resolver::resolve_module(&js_lib, &current_dir);
    match export {
        Ok(exports) => {
            imports_resolver::resolve_imports(&js_lib, &exports, &current_dir);
        }
        Err(e) => {
            println!("{}", e)
        }
    }
}
