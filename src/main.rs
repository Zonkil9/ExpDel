use std::fs;
use std::io;
use std::path;
use std::time;
use chrono::{DateTime, Local};

fn main() {
    println!("Please enter the path");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let path = path::Path::new(input.trim());

    list_by_date(path)
        .unwrap_or_else(|err| eprintln!("Error: {}", err));
}

fn list_by_date(path: &path::Path) -> io::Result<()> {
    println!("Opening {}", path.display());

    let mut entries: Vec<path::PathBuf> = fs::read_dir(&path)?
        .filter_map(|res| res.ok().map(|e| e.path()))
        .filter(|p| fs::metadata(p).is_ok())
        .collect();

    entries.sort_by_key(|k| {
        fs::metadata(k)
            .and_then(|m| m.modified())
            .ok()
    });

    for entry in entries {
        let metadata = fs::metadata(&entry)?;
        let modified: time::SystemTime = metadata.modified()?;
        let datetime: DateTime<Local> = modified.into();
        println!("Name: {} | Date: {}", entry.display(), datetime.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}