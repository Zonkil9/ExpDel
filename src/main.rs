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

    println!("\nSorting with: mtime / ctime / atime");
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

    println!("\nPlease enter how many files are NOT to be deleted");
    let mut files_to_keep = String::new();
    io::stdin()
        .read_line(&mut files_to_keep)
        .expect("Failed to read line");
    let files_to_keep: u32 = files_to_keep.trim().parse().unwrap_or(0);

    let path = path::Path::new(input.trim());

    exp_sort_and_list_to_del(&path, &sort_type, files_to_keep)
        .unwrap_or_else(|err| eprintln!("Error: {}", err));
}

fn get_time_type(meta: &fs::Metadata, sort_type: &SortType) -> time::SystemTime {
    match sort_type {
        SortType::MTime => meta.modified().unwrap_or(time::UNIX_EPOCH),
        SortType::ATime => meta.accessed().unwrap_or(time::UNIX_EPOCH),
        SortType::CTime => meta.created().unwrap_or(time::UNIX_EPOCH),
    }
}

fn exp_sort_and_list_to_del(
    path: &path::Path,
    sort_type: &SortType,
    files_to_keep: u32,
) -> io::Result<()> {
    println!("\nOpening {} and sorting by {:?}", path.display(), sort_type);

    let now = time::SystemTime::now();
    let mut groups: collections::BTreeMap<u64, Vec<(path::PathBuf, time::SystemTime)>> = collections::BTreeMap::new();
    let mut max_days = 1;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let file_time = get_time_type(&meta, &sort_type);
        if let Ok(age) = now.duration_since(file_time) {
            let days = age.as_secs() / 86400;
            let mut bucket = 1;
            while bucket <= days {
                bucket *= 2;
            }
            max_days = max_days.max(bucket);
            groups.entry(bucket).or_default().push((entry.path(), file_time));
        }
    }

    let mut bucket = 1;
    while bucket <= max_days {
        if let Some(files) = groups.get(&bucket) {
            println!("\nYounger than {} days:", bucket);
            let mut files_sorted = files.clone();
            files_sorted.sort_by_key(|(_, t)| *t);
            for (i, (file, time)) in files_sorted.iter().enumerate() {
                let datetime: chrono::DateTime<chrono::Local> = (*time).into();
                println!(
                    "{} | {}{}",
                    file.display(),
                    datetime.format("%Y-%m-%d %H:%M:%S"),
                    if (i as u32) >= files_to_keep {
                        " <-- to be deleted"
                    } else {
                        ""
                    }
                );
            }
        }
        bucket *= 2;
    }

    Ok(())
}