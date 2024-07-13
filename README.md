# filecat

`filecat` is a command-line tool for printing file contents with titles.

```shell
filecat: print file contents with headers

Usage: filecat [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  File or directory paths

Options:
  -r, --recursive        Recursively read directories
  -e, --exclude <PATH>   Exclude specific files or directories
      --header <HEADER>  Custom header format [default: "==> {file}"]
  -v, --verbose          Do not show non-printable characters
      --hex              Print file contents in hexadecimal format
  -h, --help             Print help
  -V, --version          Print version
```

## Features

- Print contents of files with customizable headers.
- Recursively read directories.
- Exclude specific files or directories from processing.
- Display non-printable characters by default, with an option to turn this off.
- Print file contents in hexadecimal format.

## Usage

To print the contents of a file with a header, simply run `filecat` with the file path as an argument:

```shell
filecat file.txt
```

To print the contents of multiple files, provide multiple file paths:

```shell
filecat file1.txt file2.txt file3.txt
```

To print the contents of a directory, use the `-r` flag:

```shell
filecat -r directory
```

To exclude specific files or directories from processing, use the `-e` flag:

```shell
filecat -e file.txt directory
```

## Installation

### crates.io

You can install `filecat` from [crates.io](https://crates.io/crates/filecat) using `cargo`:

```shell
cargo install filecat
```

### Building from Source

First, ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. Then, clone the repository and build the project:

```sh
git clone https://github.com/yourusername/filecat.git
cd filecat
cargo build --release
```

The binary will be located at `target/release/filecat`.
