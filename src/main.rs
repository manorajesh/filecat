use clap::Parser;
use colored::*;
use std::collections::HashSet;
use std::fs;
use std::io::{ self, Read, Write };
use std::path::{ Path, PathBuf };

/// Macro to print error messages with "error" colored red or "warning" colored yellow
macro_rules! print_error {
    (
        $use_color:expr,
        $($arg:tt)*
    ) => {
        if $use_color {
            eprintln!("[{}] {}", "error".red(), format!($($arg)*));
        } else {
            eprintln!("[error] {}", format!($($arg)*));
        }
    };
}

macro_rules! print_warning {
    (
        $use_color:expr,
        $($arg:tt)*
    ) => {
        if $use_color {
            eprintln!("[{}] {}", "warning".yellow(), format!($($arg)*));
        } else {
            eprintln!("[warning] {}", format!($($arg)*));
        }
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

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Write output to a file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

struct FileCat {
    header: String,
    verbose: bool,
    hex: bool,
    use_color: bool,
    output: Option<PathBuf>,
}

impl FileCat {
    fn new(
        header: String,
        verbose: bool,
        hex: bool,
        use_color: bool,
        output: Option<PathBuf>
    ) -> Self {
        FileCat { header, verbose, hex, use_color, output }
    }

    fn process_path(
        &self,
        path: &Path,
        recursive: bool,
        exclude_set: &HashSet<PathBuf>,
        output: &mut Box<dyn Write>
    ) -> io::Result<()> {
        if path.is_dir() {
            self.process_dir(path, recursive, exclude_set, output)
        } else if path.is_file() && !exclude_set.contains(path) {
            self.process_file(path, output)
        } else {
            print_error!(self.use_color, "{} is not a valid file or directory", path.display());
            Ok(())
        }
    }

    fn process_dir(
        &self,
        dir: &Path,
        recursive: bool,
        exclude_set: &HashSet<PathBuf>,
        output: &mut Box<dyn Write>
    ) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if exclude_set.contains(&path) {
                continue;
            }
            if path.is_file() {
                self.process_file(&path, output)?;
            } else if recursive && path.is_dir() {
                self.process_dir(&path, recursive, exclude_set, output)?;
            }
        }
        Ok(())
    }

    fn process_file(&self, file: &Path, output: &mut Box<dyn Write>) -> io::Result<()> {
        let mut file_content = Vec::new();
        fs::File::open(file)?.read_to_end(&mut file_content)?;
        let header = self.header.replace("{file}", &file.display().to_string());

        if self.use_color {
            writeln!(output, "{}", header.blue().bold())?;
        } else {
            writeln!(output, "{}", header)?;
        }

        if self.hex {
            self.print_hex(&file_content, output)?;
        } else {
            self.print_content(&file_content, output)?;
        }

        Ok(())
    }

    fn print_hex(&self, content: &[u8], output: &mut Box<dyn Write>) -> io::Result<()> {
        for (i, byte) in content.iter().enumerate() {
            if i % 16 == 0 {
                if i != 0 {
                    writeln!(output)?;
                }
                write!(output, "{:08x}  ", i)?;
            }
            write!(output, "{:02x} ", byte)?;
        }
        writeln!(output)
    }

    fn print_content(&self, content: &[u8], output: &mut Box<dyn Write>) -> io::Result<()> {
        if self.verbose {
            write!(output, "{}", String::from_utf8_lossy(content))?;
        } else {
            for &byte in content {
                if
                    byte.is_ascii_graphic() ||
                    byte == b'\n' ||
                    byte == b'\t' ||
                    byte == b' ' ||
                    byte == b'\r'
                {
                    write!(output, "{}", byte as char)?;
                } else {
                    write!(output, "{:?} ", byte as char)?;
                }
            }
            writeln!(output)?;
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let exclude_set: HashSet<PathBuf> = args.exclude.iter().map(PathBuf::from).collect();

    if !args.header.contains("{file}") {
        print_warning!(true, "Header does not contain the placeholder {{file}}");
    }

    if args.paths.is_empty() {
        print_error!(true, "No files or directories provided");
        return Ok(());
    }

    let viewer = FileCat::new(
        args.header,
        args.verbose,
        args.hex,
        !args.no_color,
        args.output.clone()
    );

    let mut output: Box<dyn Write> = if let Some(output_path) = &args.output {
        Box::new(fs::File::create(output_path)?)
    } else {
        Box::new(io::stdout())
    };

    for path in &args.paths {
        let path = Path::new(path);
        viewer.process_path(path, args.recursive, &exclude_set, &mut output)?;
    }

    Ok(())
}
