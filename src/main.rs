use std::fs;
use std::path::Path;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    recursive: bool,

    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,
}

fn main() {
    let args = Args::parse();
    for path in &args.paths {
        // Check if the provided path is a file or a directory
        if Path::new(path).is_file() {
            rename_file(path);
        } else if Path::new(path).is_dir() {
            rename_files_in_dir(path, args.recursive);
        } else {
            eprintln!("Error: {} is not a file or directory", path);
            std::process::exit(1);
        }
    }
}

fn rename_file(path: &str) {
    // Rename the file if it has a space in its name
    if path.contains(" ") {
        let new_path = path.replace(" ", "-");
        fs::rename(path, new_path).expect("Error renaming file");
    }
}

fn rename_files_in_dir(dir: &str, recursive: bool) {
    // Iterate through the directory and rename files with spaces in their names
    for entry in fs::read_dir(dir).expect("Error reading directory") {
        let entry = entry.expect("Error reading directory entry");
        let path = entry.path();
        if path.is_file() {
            rename_file(&path.to_string_lossy());
        } else if recursive && path.is_dir() {
            rename_files_in_dir(&path.to_string_lossy(), true);
        }
    }
}
