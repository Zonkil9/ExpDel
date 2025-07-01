use chrono;
use std::fs;
use std::io;
use std::path;
use std::time;
use std::collections;

#[derive(Debug)]
enum SortType {
    MTime,
    CTime,
    ATime,
} 

fn main() {
    println!("Please enter the path");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    println!("Sorting with: mtime / ctime / atime");
    let mut sort_type = String::new();
    io::stdin()
        .read_line(&mut sort_type)
        .expect("Failed to read line");

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

    exp_sort(path, sort_type).unwrap_or_else(|err| eprintln!("Error: {}", err));
}

fn get_time_type(meta: &fs::Metadata, sort_type: &SortType) -> time::SystemTime {
    match sort_type {
        SortType::MTime => meta.modified().unwrap_or(time::UNIX_EPOCH),
        SortType::ATime => meta.accessed().unwrap_or(time::UNIX_EPOCH),
        SortType::CTime => meta.created().unwrap_or(time::UNIX_EPOCH),
    }
}

fn exp_sort(path: &path::Path, sort_type: SortType) -> io::Result<()> {
    println!("Opening {} and sorting by {:?}", path.display(), sort_type);

    let now = time::SystemTime::now();
    let mut groups: collections::BTreeMap<u64, Vec<path::PathBuf>> = collections::BTreeMap::new();
    let mut max_days = 1;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let file_time = get_time_type(&meta, &sort_type);
        let age = now.duration_since(file_time).unwrap_or(time::Duration::ZERO);
        let days = age.as_secs() / 86400;

        let mut bucket = 1;
        while bucket <= days {
            bucket *= 2;
        }
        max_days = max_days.max(bucket);
        groups.entry(bucket).or_default().push(entry.path());
    }

    let mut bucket = 1;
    while bucket <= max_days {
        if let Some(files) = groups.get(&bucket) {
            println!("\nYounger than {} days", bucket);
            for file in files {
                let meta = fs::metadata(file)?;
                let time = get_time_type(&meta, &sort_type);
                let datetime: chrono::DateTime<chrono::Local> = time.into();
                println!("{} | {}", file.display(), datetime.format("%Y-%m-%d %H:%M:%S"));
            }
        }
        bucket *= 2;
    }

    Ok(())
}