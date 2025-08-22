# ExpDel
The **ExpDel** is a tool for deleting files <ins>exponentially</ins> based on their times in a specified path. Specifically, it works as follows:

1. The program divides the time interval into exponentially increasing time segments
2. It asks the user how many of the oldest files in each time segment to keep 
3. It deletes the rest

It is designed to be used in a terminal environment and it was written in Rust.
It is particularly useful for managing large directories with many files, where you want to keep a certain number of the oldest files in each time segment while deleting the rest.

The program was tested **only on Linux**. It might work on other operating systems, but this is not guaranteed.

# Important Note
**You delete files at your own risk.** This tool does not move files to a recycle bin or trash; it permanently deletes them. Always ensure you have backups of important data before using this tool.

# Installation
To install ExpDel, you need to have Rust installed on your system. Then, you can clone the repository and build the project:

```bash
git clone https://github.com/Zonkil9/ExpDel
cd ExpDel
cargo build --release
```

# Usage
After building the project, you can run the executable from the `./target/release` directory:
```bash
./target/release/ExpDel
```

You can also run it directly using Cargo:
```bash
cargo run --release
```

Then, follow the prompts to specify the path and how many oldest files to keep in each time segment. The program will list the files that would be deleted based on the specified criteria.

# Testing
To run the tests, you can use the following command:
```bash
cargo test
```
