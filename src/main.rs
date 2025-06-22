use std::fs;
use std::io;
use std::path;
use std::time;
use chrono::{DateTime, Local};

#[derive(Debug)]
enum SortType {
    MTime,
    CTime,
    ATime,
}

fn main() {
    println!("Please enter the path");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    println!("Sorting with: mtime / ctime / atime");
    let mut sort_type = String::new();
    io::stdin().read_line(&mut sort_type).expect("Failed to read line");

    let sort_type = match sort_type.trim() {
        "mtime" => SortType::MTime,
        "ctime" => SortType::CTime,
        "atime" => SortType::ATime,
        _ => {
            eprintln!("Invalid sort type. Defaulting to mtime.");
            SortType::MTime
        }
    };

    let path = path::Path::new(input.trim());

    list_by_date(path, sort_type)
        .unwrap_or_else(|err| eprintln!("Error: {}", err));
}

fn list_by_date(path: &path::Path, sort_type: SortType) -> io::Result<()> {
    println!("Opening {} and sorting by {:?}", path.display(), sort_type);

    let mut entries: Vec<path::PathBuf> = fs::read_dir(&path)?
        .filter_map(|res| res.ok().map(|e| e.path()))
        .filter(|p| fs::metadata(p).is_ok())
        .collect();

    entries.sort_by_key(|k| {
        fs::metadata(k)
            .map(|m| get_time_type(&m, &sort_type))
            .unwrap_or(time::UNIX_EPOCH)
    });

    for entry in entries {
        let metadata = fs::metadata(&entry)?;
        let time = get_time_type(&metadata, &sort_type);
        let datetime: DateTime<Local> = time.into();
        println!("Name: {} | Date: {}", entry.display(), datetime.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}

fn get_time_type(meta: &fs::Metadata, sort_type: &SortType) -> time::SystemTime {
    match sort_type {
        SortType::MTime => meta.modified().unwrap_or(time::UNIX_EPOCH),
        SortType::ATime => meta.accessed().unwrap_or(time::UNIX_EPOCH),
        SortType::CTime => meta.created().unwrap_or(time::UNIX_EPOCH),
    }
}