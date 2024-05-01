use clap::Parser;
use eyre::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/git_describe.rs"));
}

#[derive(Parser, Debug)]
#[command(name = "dashify", about = "lowercases, and removes spaces and other chars in file names")]
#[command(version = built_info::GIT_DESCRIBE)]
#[command(author = "Scott A. Idler <scott.a.idler@gmail.com>")]
#[command(arg_required_else_help = true)]
struct Args {
    #[arg(short, long)]
    recursive: bool,

    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    for path in &args.paths {
        if Path::new(path).is_file() {
            rename_file(path)?;
        } else if Path::new(path).is_dir() {
            rename_files_in_dir(path, args.recursive)?;
        } else {
            eprintln!("Error: {path} is not a file or directory");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn rename_file(path: &str) -> Result<()> {
    let re = Regex::new(r"[, ]+")?;
    let new_path = re.replace_all(path, "-");

    let re_hyphens = Regex::new(r"-+")?;
    let new_path = re_hyphens.replace_all(&new_path, "-");

    let new_path = new_path.to_lowercase();
    fs::rename(path, new_path.as_ref() as &std::path::Path)?;
    Ok(())
}


fn rename_files_in_dir(dir: &str, recursive: bool) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            rename_file(&path.to_string_lossy())?;
        } else if recursive && path.is_dir() {
            rename_files_in_dir(&path.to_string_lossy(), true)?;
        }
    }
    Ok(())
}
