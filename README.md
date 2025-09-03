# ExpDel

The **ExpDel** is a CLI tool for deleting files <ins>exponentially</ins> based on their times in a specified path.
Specifically, the program:

1. Divides the files into exponentially increasing time segments (i.e., 2^0, 2^1, 2^2, 2^3, ... days)
2. Asks the user how many of the oldest files in each time segment to keep (e.g., keep 2 oldest files in each segment)
3. Deletes the rest. *Forever*.

The program was written in Rust.
It is particularly useful if you want to keep a certain number of the oldest files in each time segment while deleting
the rest, e.g. working with log files or backups.

The program was tested **only on Linux**. It might work on other operating systems, but this is not guaranteed.

# Important Note

**You delete files at your own risk.** This tool does not move files to a recycle bin or trash; it permanently deletes
them. Always ensure you have backups of important data before using this tool.

# Installation

To compile ExpDel, you need to have Rust installed on your system. Then, you can clone the repository and build the
project:

```bash
git clone https://github.com/Zonkil9/ExpDel
cd ExpDel
cargo build --release
```

# Usage

After building the project, you can run the executable from the `./target/release` directory. For example:

```bash
./target/release/ExpDel --path /path/to/directory --keep 2
```

Try `./target/release/ExpDel --help` for more information on usage and options.

# Testing

To run the tests, you can use the following command in the project directory:

```bash
cargo test
```

# Future Plans

- [ ] Add more options for specifying time segments (e.g., weekly, monthly)
- [ ] Add an option to delete the youngest files in each segment instead of the oldest
- [ ] Filter files by type or extension
- [ ] Different exponential bases (e.g., base 3, base 10)
