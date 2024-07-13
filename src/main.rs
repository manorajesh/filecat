use clap::Parser;
use colored::*;
use std::collections::HashSet;
use std::fs;
use std::io::{ self, Read };
use std::path::{ Path, PathBuf };

/// Macro to print error messages with "error" colored red or "warning" colored yellow
macro_rules! print_error {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", "error".red(), format!($($arg)*));
    };
}

macro_rules! print_warning {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", "warning".yellow(), format!($($arg)*));
    };
}

/// filecat: print file contents with headers
#[derive(Parser, Debug)]
#[command(name = "filecat", author, version, about, long_about = None)]
struct Args {
    /// File or directory paths
    paths: Vec<String>,

    /// Recursively read directories
    #[arg(short, long)]
    recursive: bool,

    /// Exclude specific files or directories
    #[arg(short, long, value_name = "PATH")]
    exclude: Vec<String>,

    /// Custom header format
    #[arg(long, default_value = "==> {file}")]
    header: String,

    /// Do not show non-printable characters
    #[arg(short, long)]
    verbose: bool,

    /// Print file contents in hexadecimal format
    #[arg(long)]
    hex: bool,
}

struct FileCat {
    header: String,
    verbose: bool,
    hex: bool,
}

impl FileCat {
    fn new(header: String, verbose: bool, hex: bool) -> Self {
        FileCat { header, verbose, hex }
    }

    fn process_path(
        &self,
        path: &Path,
        recursive: bool,
        exclude_set: &HashSet<PathBuf>
    ) -> io::Result<()> {
        if path.is_dir() {
            self.process_dir(path, recursive, exclude_set)
        } else if path.is_file() && !exclude_set.contains(path) {
            self.process_file(path)
        } else {
            print_error!("{} is not a valid file or directory", path.display());
            Ok(())
        }
    }

    fn process_dir(
        &self,
        dir: &Path,
        recursive: bool,
        exclude_set: &HashSet<PathBuf>
    ) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if exclude_set.contains(&path) {
                continue;
            }
            if path.is_file() {
                self.process_file(&path)?;
            } else if recursive && path.is_dir() {
                self.process_dir(&path, recursive, exclude_set)?;
            }
        }
        Ok(())
    }

    fn process_file(&self, file: &Path) -> io::Result<()> {
        let mut file_content = Vec::new();
        fs::File::open(file)?.read_to_end(&mut file_content)?;
        let header = self.header.replace("{file}", &file.display().to_string());
        println!("{}", header.blue().bold());

        if self.hex {
            self.print_hex(&file_content);
        } else {
            self.print_content(&file_content);
        }

        Ok(())
    }

    fn print_hex(&self, content: &[u8]) {
        for (i, byte) in content.iter().enumerate() {
            if i % 16 == 0 {
                if i != 0 {
                    println!();
                }
                print!("{:08x}  ", i);
            }
            print!("{:02x} ", byte);
        }
        println!();
    }

    fn print_content(&self, content: &[u8]) {
        if self.verbose {
            print!("{}", String::from_utf8_lossy(content));
        } else {
            for &byte in content {
                if
                    byte.is_ascii_graphic() ||
                    byte == b'\n' ||
                    byte == b'\t' ||
                    byte == b' ' ||
                    byte == b'\r'
                {
                    print!("{}", byte as char);
                } else {
                    print!("{:?} ", byte as char);
                }
            }
            println!();
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let exclude_set: HashSet<PathBuf> = args.exclude.iter().map(PathBuf::from).collect();

    if !args.header.contains("{file}") {
        print_warning!("Header does not contain the placeholder {{file}}");
    }

    if args.paths.is_empty() {
        print_error!("No files or directories provided");
        return Ok(());
    }

    let viewer = FileCat::new(args.header, args.verbose, args.hex);

    for path in &args.paths {
        let path = Path::new(path);
        viewer.process_path(path, args.recursive, &exclude_set)?;
    }

    Ok(())
}
