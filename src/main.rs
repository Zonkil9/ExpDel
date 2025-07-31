use chrono;
use itertools::Itertools;
use std::collections;
use std::fs;
use std::io;
use std::path;
use std::time;

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

    exp_sort_and_list_to_del(&path, &sort_type, files_to_keep).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        (Vec::new(), Vec::new())
    });
}

fn get_time_type(meta: &fs::Metadata, sort_type: &SortType) -> time::SystemTime {
    match sort_type {
        SortType::MTime => meta.modified().unwrap_or(time::UNIX_EPOCH),
        SortType::ATime => meta.accessed().unwrap_or(time::UNIX_EPOCH),
        SortType::CTime => meta.created().unwrap_or(time::UNIX_EPOCH),
    }
}

fn group_files_by_bucket(
    path: &path::Path,
    sort_type: &SortType,
) -> io::Result<collections::BTreeMap<u64, Vec<(path::PathBuf, time::SystemTime)>>> {
    let now = time::SystemTime::now();
    let mut groups: collections::BTreeMap<u64, Vec<(path::PathBuf, time::SystemTime)>> =
        collections::BTreeMap::new();

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The provided path does not exist.",
        ));
    }
    if path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The provided path is a file, not a directory.",
        ));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if !meta.is_file() {
            continue; // Skip directories and other non-file entries
        }
        let file_time = get_time_type(&meta, &sort_type);
        if let Ok(age) = now.duration_since(file_time) {
            let days = age.as_secs() / 86400;
            let bucket = if days == 0 {
                1
            } else {
                1 << (days.checked_ilog2().unwrap() + if days.is_power_of_two() { 0 } else { 1 })
            };
            groups
                .entry(bucket)
                .or_default()
                .push((entry.path(), file_time));
        }
    }
    if groups.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No files found in the directory. Remember that the program only works with files, not directories.",
        ));
    }
    Ok(groups)
}

fn exp_sort_and_list_to_del(
    path: &path::Path,
    sort_type: &SortType,
    files_to_keep: u32,
) -> io::Result<(Vec<path::PathBuf>, Vec<path::PathBuf>)> {
    println!(
        "\nOpening {}, sorting by {:?} and keeping {} files",
        path.display(),
        sort_type,
        files_to_keep
    );

    let groups = group_files_by_bucket(path, sort_type)?;
    let mut to_keep = Vec::new();
    let mut to_delete = Vec::new();

    if files_to_keep == 0 && !cfg!(test) {
        println!("No files will be kept, you want ALL files to be deleted.");
        println!("Are you sure you want to proceed? (yes/no)");
        let mut confirmation = String::new();
        io::stdin()
            .read_line(&mut confirmation)
            .expect("Failed to read line");
        if confirmation.trim().to_lowercase() != "yes" {
            println!("Operation cancelled.");
            return Ok((Vec::new(), Vec::new()));
        }
    } else if files_to_keep == 0 && cfg!(test) {
        println!("(Test mode) Skipping confirmation.");
    }

    for (bucket, files) in groups.iter() {
        println!(
            "\nYounger than {} days but older than {} days:",
            bucket,
            bucket / 2
        );
        let sorted: Vec<_> = files.iter().sorted_by_key(|(_, t)| *t).collect();
        let split_idx = files_to_keep.min(sorted.len() as u32) as usize; // Ensure the code doesn't panic
        let (keep, delete) = sorted.split_at(split_idx); // Split the sorted files into two groups

        if delete.is_empty() {
            println!("No files to delete in this group.");
        }

        for (file, time) in keep {
            let datetime: chrono::DateTime<chrono::Local> = (*time).into();
            println!(
                "{} | {}",
                file.display(),
                datetime.format("%Y-%m-%d %H:%M:%S")
            );
            to_keep.push(file.clone());
        }
        for (file, time) in delete {
            let datetime: chrono::DateTime<chrono::Local> = (*time).into();
            println!(
                "{} | {} <-- to be deleted",
                file.display(),
                datetime.format("%Y-%m-%d %H:%M:%S")
            );
            to_delete.push(file.clone());
        }
    }

    Ok((to_keep, to_delete))
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{FileTime, set_file_times};
    use rand::Rng;
    use std::io::Write;
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn test_get_time_type() {
        println!("Testing get_time_type function");

        let meta = fs::metadata("Cargo.toml").unwrap();
        let mtime = get_time_type(&meta, &SortType::MTime);
        let atime = get_time_type(&meta, &SortType::ATime);
        let ctime = get_time_type(&meta, &SortType::CTime);

        assert!(mtime > time::UNIX_EPOCH);
        assert!(atime > time::UNIX_EPOCH);
        assert!(ctime > time::UNIX_EPOCH);
    }

    #[test]
    fn test_simple() {
        println!("Testing a normal directory structure");

        let dir = tempdir().unwrap();
        let mut rng = rand::rng();

        for i in 0..500 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "test {}", i).unwrap();

            let now = time::SystemTime::now();
            let offset_secs = rng.random_range(0..365 * 24 * 3600);
            let random_time = FileTime::from_unix_time(
                now.duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64 - offset_secs as i64,
                0,
            );

            set_file_times(&file_path, random_time, random_time).unwrap();
        } // Create some files with different times, max one-year-old

        let result = exp_sort_and_list_to_del(dir.path(), &SortType::MTime, rng.random_range(1..5));
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::ATime, rng.random_range(1..5));
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::CTime, rng.random_range(1..5)); //Can't modify ctime in tests so always one bucket
        assert!(result.is_ok());
    }

    #[test]
    fn test_files_to_delete_are_correct() {
        println!("Testing that files to delete are correct");

        let dir = tempdir().unwrap();
        let file1 = dir.path().join("oldest.txt");
        let file2 = dir.path().join("youngest.txt");
        let file3 = dir.path().join("second_youngest.txt");
        let file4 = dir.path().join("third_youngest.txt");
        fs::File::create(&file1).unwrap();
        fs::File::create(&file2).unwrap();
        fs::File::create(&file3).unwrap();
        fs::File::create(&file4).unwrap();

        let now = time::SystemTime::now();
        set_file_times(
            &file1,
            FileTime::from_system_time(now - time::Duration::from_secs(10000)),
            FileTime::from_system_time(now - time::Duration::from_secs(10000)),
        )
        .unwrap();
        set_file_times(
            &file2,
            FileTime::from_system_time(now),
            FileTime::from_system_time(now),
        )
        .unwrap();
        set_file_times(
            &file3,
            FileTime::from_system_time(now - time::Duration::from_secs(1)),
            FileTime::from_system_time(now - time::Duration::from_secs(1)),
        )
        .unwrap();
        set_file_times(
            &file4,
            FileTime::from_system_time(now - time::Duration::from_secs(500)),
            FileTime::from_system_time(now - time::Duration::from_secs(500)),
        )
        .unwrap();

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 1).unwrap();

        assert!(to_keep.contains(&file1));
        assert!(to_delete.contains(&file3));
        assert!(to_delete.contains(&file4));
        assert!(to_delete.contains(&file2));
        assert_eq!(to_keep.len(), 1);
        assert_eq!(to_delete.len(), 3);

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(dir.path(), &SortType::ATime, 1).unwrap();
        assert!(to_keep.contains(&file1));
        assert!(to_delete.contains(&file3));
        assert!(to_delete.contains(&file4));
        assert!(to_delete.contains(&file2));
        assert_eq!(to_keep.len(), 1);
        assert_eq!(to_delete.len(), 3);

        //Ctime is tested separately since it cannot be easily modified in tests
    }

    #[test]
    fn test_ctime() {
        println!("Testing ctime sorting");

        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        fs::File::create(&file1).unwrap();

        thread::sleep(time::Duration::from_secs(2)); // Ensure a difference in ctime

        let file2 = dir.path().join("file2.txt");
        fs::File::create(&file2).unwrap();

        thread::sleep(time::Duration::from_secs(2));

        let file3 = dir.path().join("file3.txt");
        fs::File::create(&file3).unwrap();

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(dir.path(), &SortType::CTime, 1).unwrap();

        assert!(to_keep.contains(&file1));
        assert!(to_delete.contains(&file2));
        assert!(to_delete.contains(&file3));
        assert_eq!(to_keep.len(), 1);
        assert_eq!(to_delete.len(), 2);
    }

    #[test]
    fn test_buckets_behavior() {
        println!("Testing buckets behavior explicitly");

        let dir = tempdir().unwrap();
        let now = time::SystemTime::now();

        for i in 0..16 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            fs::File::create(&file_path).unwrap();
            let days = i * 86400;
            set_file_times(
                &file_path,
                FileTime::from_system_time(now - time::Duration::from_secs(days)),
                FileTime::from_system_time(now - time::Duration::from_secs(days)),
            )
            .unwrap();
        }

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 1).unwrap();

        assert!(to_delete.contains(&dir.path().join("file0.txt"))); //Files asserted explicitly
        assert!(to_keep.contains(&dir.path().join("file1.txt")));
        assert!(to_keep.contains(&dir.path().join("file2.txt")));
        assert!(to_delete.contains(&dir.path().join("file3.txt")));
        assert!(to_keep.contains(&dir.path().join("file4.txt")));
        assert!(to_delete.contains(&dir.path().join("file5.txt")));
        assert!(to_delete.contains(&dir.path().join("file6.txt")));
        assert!(to_delete.contains(&dir.path().join("file7.txt")));
        assert!(to_keep.contains(&dir.path().join("file8.txt")));
        assert!(to_delete.contains(&dir.path().join("file9.txt")));
        assert!(to_delete.contains(&dir.path().join("file10.txt")));
        assert!(to_delete.contains(&dir.path().join("file11.txt")));
        assert!(to_delete.contains(&dir.path().join("file12.txt")));
        assert!(to_delete.contains(&dir.path().join("file13.txt")));
        assert!(to_delete.contains(&dir.path().join("file14.txt")));
        assert!(to_keep.contains(&dir.path().join("file15.txt")));
        assert_eq!(to_keep.len(), 5);
        assert_eq!(to_delete.len(), 11);

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(dir.path(), &SortType::ATime, 1).unwrap();

        assert!(to_delete.contains(&dir.path().join("file0.txt")));
        assert!(to_keep.contains(&dir.path().join("file1.txt")));
        assert!(to_keep.contains(&dir.path().join("file2.txt")));
        assert!(to_delete.contains(&dir.path().join("file3.txt")));
        assert!(to_keep.contains(&dir.path().join("file4.txt")));
        assert!(to_delete.contains(&dir.path().join("file5.txt")));
        assert!(to_delete.contains(&dir.path().join("file6.txt")));
        assert!(to_delete.contains(&dir.path().join("file7.txt")));
        assert!(to_keep.contains(&dir.path().join("file8.txt")));
        assert!(to_delete.contains(&dir.path().join("file9.txt")));
        assert!(to_delete.contains(&dir.path().join("file10.txt")));
        assert!(to_delete.contains(&dir.path().join("file11.txt")));
        assert!(to_delete.contains(&dir.path().join("file12.txt")));
        assert!(to_delete.contains(&dir.path().join("file13.txt")));
        assert!(to_delete.contains(&dir.path().join("file14.txt")));
        assert!(to_keep.contains(&dir.path().join("file15.txt")));
        assert_eq!(to_keep.len(), 5);
        assert_eq!(to_delete.len(), 11);

        // CTime is not tested here since it cannot be easily modified in tests
    }

    #[test]
    fn test_identical_times() {
        println!("Testing with files having identical modification times");

        let dir = tempdir().unwrap();
        let now = time::SystemTime::now();
        let ft = FileTime::from_system_time(now);

        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        let file3 = dir.path().join("file3.txt");
        let file4 = dir.path().join("file4.txt");
        fs::File::create(&file1).unwrap();
        fs::File::create(&file2).unwrap();
        fs::File::create(&file3).unwrap();
        fs::File::create(&file4).unwrap();
        set_file_times(&file1, ft, ft).unwrap();
        set_file_times(&file2, ft, ft).unwrap();
        set_file_times(&file3, ft, ft).unwrap();
        set_file_times(&file4, ft, ft).unwrap();

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 2).unwrap(); //Function deletes randomly. It is expected behavior for now. Maybe change in the future for asking the user.

        assert_eq!(to_keep.len(), 2);
        assert_eq!(to_delete.len(), 2);
        assert_eq!(to_keep.len() + to_delete.len(), 4);
    }

    #[test]
    fn test_zero_files_to_keep() {
        println!("Testing with zero files to keep");

        let dir = tempdir().unwrap();
        let mut rng = rand::rng();

        for i in 0..5 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "test {}", i).unwrap();

            let now = time::SystemTime::now();
            let offset_secs = rng.random_range(0..30 * 24 * 3600);
            let random_time = FileTime::from_unix_time(
                now.duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64 - offset_secs as i64,
                0,
            );

            set_file_times(&file_path, random_time, random_time).unwrap();
        }

        let result = exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 0);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::ATime, 0);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::CTime, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_directory() {
        println!("Testing with an empty directory");

        let dir = tempdir().unwrap();
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_path() {
        println!("Testing with an invalid path");

        let invalid_path = path::Path::new("/invalid/path");
        let result = exp_sort_and_list_to_del(invalid_path, &SortType::MTime, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_instead_of_directory() {
        println!("Testing with a file instead of a directory");

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        fs::File::create(&file_path).unwrap();
        let result = exp_sort_and_list_to_del(&file_path, &SortType::MTime, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_same_times() {
        println!("Testing with files having the same modification time");

        //For now the program deletes randomly
        //Maybe different implementation in the future

        let dir = tempdir().unwrap();
        let now = time::SystemTime::now();
        let ft = FileTime::from_system_time(now);
        for i in 0..3 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            fs::File::create(&file_path).unwrap();
            set_file_times(&file_path, ft, ft).unwrap();
        }

        let result = exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 1);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::ATime, 1);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(dir.path(), &SortType::CTime, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_directory_with_subdirectories() {
        println!("Testing with a directory containing subdirectories");

        let dir = tempdir().unwrap();
        for i in 0..5 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            fs::File::create(&file_path).unwrap();
        }
        let sub_dir_path = dir.path().join("sub_dir");
        fs::create_dir(&sub_dir_path).unwrap();
        let subfile_path = sub_dir_path.join("subfile.txt");
        fs::File::create(&subfile_path).unwrap();

        let result = exp_sort_and_list_to_del(dir.path(), &SortType::MTime, 1);
        assert!(result.is_ok());
    }
}
