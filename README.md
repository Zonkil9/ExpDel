# ExpDel

The **ExpDel** is a CLI tool for deleting files <ins>exponentially</ins> based on their times in a specified path.
Specifically, the program:

1. Divides the files into exponentially increasing time segments (i.e., 2^0, 2^1, 2^2, 2^3, ... days)
2. Asks the user how many of the oldest files in each time segment to keep (e.g., keep 2 oldest files in each segment)
3. Deletes the rest. *Forever*.

The program was written in Rust.
It is particularly useful if you want to keep a certain number of the oldest files in each time segment while deleting
the rest, e.g. working with log files or backups.

The program was tested mainly on Linux. It might work on Windows and macOS, as well as other filesystems, but this is
not guaranteed.

# Important Note

**You delete files at your own risk.** This tool does not move files to a recycle bin or trash; it permanently deletes
them. Always ensure you have backups of important data before using this tool.

# Installation

You can download the latest ExpDel release from the [Releases](https://github.com/Zonkil9/ExpDel/releases) page.
Then, you can simply run the executable. See the [Usage](#usage) section for more information.

# Compilation

Alternatively, you can compile ExpDel yourself. In order to compile the project, you need to have Rust installed on your
system. Then, you can clone the repository and build the project:

```bash
git clone https://github.com/Zonkil9/ExpDel
cd ExpDel
cargo build --release
```

The compiled binary will be located in the `target/release` directory.

To run the tests, you can use the following command in the project directory:

```bash
cargo test
```

# Usage

You can run the program from the command line. The basic usage is as follows:

```bash
./ExpDel --path /path/to/directory --keep 2
```

Try `./ExpDel --help` for more information on usage and options.

# Future Plans

- [ ] Add more options for specifying time segments (e.g., weekly, monthly)
- [ ] Add an option to delete the youngest files in each segment instead of the oldest
- [ ] Filter files by type or extension
- [ ] Different exponential bases (e.g., base 3, base 10)
