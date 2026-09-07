use clap::Parser;
use expand_tilde::expand_tilde;
use eyre::Result;
use std::fs;
use std::path::Path;

use dashify::{dashify, DashifyOptions};

// Built-in version from build.rs via env!("GIT_DESCRIBE")

#[derive(Parser, Debug)]
#[command(
    name = "dashify",
    about = "Normalize filenames: converts CamelCase, spaces, and special chars to dashes"
)]
#[command(version = env!("GIT_DESCRIBE"))]
#[command(author = "Scott A. Idler <scott.a.idler@gmail.com>")]
#[command(arg_required_else_help = true)]
struct Args {
    #[arg(short, long, help = "Recursively process files in subdirectories")]
    recursive: bool,

    #[arg(short, long, help = "Show what would be renamed without actually renaming")]
    dry_run: bool,

    #[arg(short, long, help = "Force underscores to become dashes")]
    force_dash: bool,

    #[arg(
        value_name = "PATH",
        default_value = ".",
        help = "Path to file or directory to process"
    )]
    paths: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let options = DashifyOptions {
        force_dash: args.force_dash,
    };
    for path in &args.paths {
        let expanded_path = expand_tilde(path);
        if expanded_path.is_file() {
            rename_file(&expanded_path, args.dry_run, &options)?;
        } else if expanded_path.is_dir() {
            rename_files_in_dir(&expanded_path, args.recursive, args.dry_run, &options)?;
        } else {
            eprintln!("Error: {path} is not a file or directory");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn rename_file(path: &Path, dry_run: bool, options: &DashifyOptions) -> Result<()> {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        let new_file_name = dashify(&file_name, options);

        if new_file_name != file_name {
            let new_path = path.with_file_name(&new_file_name);
            if dry_run {
                println!("{} -> {}", file_name, new_file_name);
            } else {
                fs::rename(path, &new_path)?;
                println!("{} -> {}", file_name, new_file_name);
            }
        }
    }
    Ok(())
}

fn rename_files_in_dir(dir: &Path, recursive: bool, dry_run: bool, options: &DashifyOptions) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            rename_file(&path, dry_run, options)?;
        } else if recursive && path.is_dir() {
            rename_files_in_dir(&path, true, dry_run, options)?;
        }
    }
    Ok(())
}
