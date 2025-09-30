use clap::Parser;
use eyre::Result;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

// Built-in version from build.rs via env!("GIT_DESCRIBE")

#[derive(Parser, Debug)]
#[command(name = "dashify", about = "lowercases, removes spaces, underscores, and other unwanted chars in file names")]
#[command(version = env!("GIT_DESCRIBE"))]
#[command(author = "Scott A. Idler <scott.a.idler@gmail.com>")]
#[command(arg_required_else_help = true)]
struct Args {
    #[arg(short, long, help = "Recursively process files in subdirectories")]
    recursive: bool,

    #[arg(value_name = "PATH", default_value = ".", help = "Path to file or directory to process")]
    paths: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    for path in &args.paths {
        let expanded_path = expand_tilde(path);
        if Path::new(&expanded_path).is_file() {
            rename_file(&expanded_path)?;
        } else if Path::new(&expanded_path).is_dir() {
            rename_files_in_dir(&expanded_path, args.recursive)?;
        } else {
            eprintln!("Error: {path} is not a file or directory");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn expand_tilde(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        if path.starts_with("~") {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

fn rename_file(path: &str) -> Result<()> {
    let path_buf = PathBuf::from(path);
    if let Some(file_name) = path_buf.file_name() {
        let file_name = file_name.to_string_lossy();

        let re = Regex::new(r"[,_ ]|\\(|\\)")?;
        let mut new_file_name = re.replace_all(&file_name, "-").to_string();

        let re_hyphens = Regex::new(r"-+")?;
        new_file_name = re_hyphens.replace_all(&new_file_name, "-").to_string();
        new_file_name = new_file_name.trim_matches('-').to_string();
        new_file_name = new_file_name.to_lowercase();

        let new_path = path_buf.with_file_name(new_file_name);
        fs::rename(path_buf, new_path)?;
    }
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
