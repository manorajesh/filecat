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

macro_rules! print_info {
    (
        $use_color:expr,
        $($arg:tt)*
    ) => {
        if $use_color {
            eprintln!("[{}] {}", "info".bright_blue(), format!($($arg)*));
        } else {
            eprintln!("[info] {}", format!($($arg)*));
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

    /// Print non-text file contents in hexadecimal format
    #[arg(long)]
    hex: bool,

    /// Enable colored output of headers
    #[arg(long)]
    color: bool,

    /// Disable colored output of log messages
    #[arg(long)]
    no_log_color: bool,

    /// Write output to a file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Enable file counter
    #[arg(long)]
    counter: bool,

    /// Skip non-text files but still print headers
    #[arg(long)]
    skip_non_text: bool,
}

struct FileCat {
    header: String,
    verbose: bool,
    hex: bool,
    use_color: bool,
    output: Option<PathBuf>,
    counter: bool,
    skip_non_text: bool,
    file_count: usize,
    use_log_color: bool,
}

impl FileCat {
    fn new(
        header: String,
        verbose: bool,
        hex: bool,
        use_color: bool,
        output: Option<PathBuf>,
        counter: bool,
        skip_non_text: bool,
        use_log_color: bool
    ) -> Self {
        FileCat {
            header,
            verbose,
            hex,
            use_color,
            output,
            counter,
            skip_non_text,
            file_count: 0,
            use_log_color,
        }
    }

    fn process_path(
        &mut self,
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
            print_error!(self.use_log_color, "{} is not a valid file or directory", path.display());
            Ok(())
        }
    }

    fn process_dir(
        &mut self,
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

    fn process_file(&mut self, file: &Path, output: &mut Box<dyn Write>) -> io::Result<()> {
        let mut file_content = Vec::new();
        fs::File::open(file)?.read_to_end(&mut file_content)?;
        let header = self.header.replace("{file}", &file.display().to_string());

        if self.use_color {
            writeln!(output, "{}", header.blue().bold())?;
        } else {
            writeln!(output, "{}", header)?;
        }

        if !self.is_text_file(&file_content) {
            if self.skip_non_text {
                writeln!(output, "Non-text file")?;
                return Ok(());
            } else if self.hex {
                self.print_hex(&file_content, output)?;
                return Ok(());
            }
        }

        self.print_content(&file_content, output)?;

        if self.counter {
            self.file_count += 1;
            print_info!(self.use_log_color, "Files processed so far: {}", self.file_count);
        }

        Ok(())
    }

    fn is_text_file(&self, content: &[u8]) -> bool {
        content
            .iter()
            .all(|&byte| (byte.is_ascii_graphic() || byte.is_ascii_whitespace() || byte == b'\r'))
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

    let use_log_color = !args.no_log_color;
    let use_color = args.color;

    if let Some(output_path) = &args.output {
        if output_path.is_dir() {
            print_error!(use_log_color, "Output path is a directory");
            return Ok(());
        }

        if output_path.exists() {
            print_error!(use_log_color, "Output file already exists");
            return Ok(());
        }
    }

    if !args.header.contains("{file}") {
        print_warning!(use_log_color, "Header does not contain the placeholder {{file}}");
    }

    if args.paths.is_empty() {
        print_error!(use_log_color, "No files or directories provided");
        return Ok(());
    }

    let mut viewer = FileCat::new(
        args.header,
        args.verbose,
        args.hex,
        use_color,
        args.output.clone(),
        args.counter,
        args.skip_non_text,
        use_log_color
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

    if args.counter {
        print_info!(use_log_color, "Total files processed: {}", viewer.file_count);
    }

    Ok(())
}
