use chrono;
use clap::Parser;
use itertools::Itertools;
use std::collections;
use std::fs;
use std::io;
use std::path;
use std::process;
use std::time;
use walkdir::WalkDir;

/// Simple tool for deleting files exponentially based on their times in a specified path
#[derive(Parser, Debug)]
#[command(version = "0.1.1", about, author = "Zonkil9", long_about = None)]
struct Args {
    /// Path to the directory
    #[arg(short = 'p', long)]
    path: String,

    /// Sort by: mtime (modification time), ctime (creation time), atime (access time)
    #[arg(short = 's', long, default_value = "ctime")]
    sort: String,

    /// Number of files to keep per time segment
    #[arg(short = 'k', long)]
    keep: u32,

    /// FOR EXPERTS ONLY! Use with caution.
    /// Automatically confirm deletion without prompting. Cannot be used with --print_only.
    #[arg(short = 'f', long, default_value_t = false)]
    force: bool,

    ///This is a Print only mode, so-called "dry run". No files will be deleted.
    ///Cannot be used with --force or --quiet.
    #[arg(short = 'o', long, default_value_t = false)]
    print_only: bool,

    /// Recursive mode: also process files in subdirectories.
    #[arg(short = 'r', long, default_value_t = false)]
    recursive: bool,

    /// Quiet mode: no output, except for errors. Silent deletion.
    /// Cannot be used with --print_only.
    #[arg(short = 'q', long, default_value_t = false)]
    quiet: bool,
}

#[derive(Debug)]
enum SortType {
    MTime,
    CTime,
    ATime,
}

macro_rules! println_if_not_quiet {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!($($arg)*);
        }
    };
}

fn main() {
    let args = Args::parse();

    if args.quiet && args.print_only {
        eprintln!("Error: --quiet and --print_only cannot be used together.");
        process::exit(1);
    }

    if args.print_only && args.force {
        eprintln!("Error: --print_only and --force cannot be used together.");
        process::exit(1);
    }

    let path = path::Path::new(&args.path);

    if !path.exists() {
        eprintln!("Error: The provided path does not exist.");
        process::exit(1);
    }
    if path.is_file() {
        eprintln!("Error: The provided path is a file, not a directory.");
        process::exit(1);
    }

    let sort_type = match args.sort.to_lowercase().as_str() {
        "mtime" => SortType::MTime,
        "ctime" => SortType::CTime,
        "atime" => SortType::ATime,
        _ => {
            eprintln!("Invalid sort type. Defaulting to ctime.");
            SortType::CTime
        }
    };

    let (_to_keep, to_delete) =
        exp_sort_and_list_to_del(args.quiet, &path, &sort_type, args.keep, args.recursive)
            .unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
                (Vec::new(), Vec::new())
            });

    if !args.force && !args.print_only && !args.quiet && !to_delete.is_empty() {
        if _to_keep.is_empty() {
            println!("WARNING! No files will be kept, you want ALL files to be deleted.");
        }
        println!("\nDo you want to proceed with deletion? There is no undo. (yes/no)");
        let mut confirmation = String::new();
        io::stdin()
            .read_line(&mut confirmation)
            .expect("Failed to read line");
        if confirmation.trim().to_lowercase() != "yes" {
            println!("Operation cancelled.");
            return;
        }
    }

    if !args.print_only {
        if !to_delete.is_empty() {
            delete_files(args.quiet, &to_delete).unwrap_or_else(|err| {
                eprintln!("Error during deletion: {}", err);
            });
        } else {
            println!("No files to delete.");
        }
    } else {
        println!("\nPrint-only enabled, no files were deleted.");
    }
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

fn group_files_by_bucket_recursive(
    root: &path::Path,
    sort_type: &SortType,
) -> io::Result<
    collections::BTreeMap<
        path::PathBuf,
        collections::BTreeMap<u64, Vec<(path::PathBuf, time::SystemTime)>>,
    >,
> {
    let mut all_groups = collections::BTreeMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_dir() {
            let dir_path = entry.path();
            let groups = group_files_by_bucket(dir_path, sort_type)?;
            if !groups.is_empty() {
                all_groups.insert(dir_path.to_path_buf(), groups);
            } else {
                println_if_not_quiet!(
                    false,
                    "Directory {} is empty. Skipping.",
                    dir_path.display()
                );
            }
        }
    }

    if all_groups.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No files found in the directory or its subdirectories. Remember that the program only works with files, not directories.",
        ));
    }

    Ok(all_groups)
}

fn exp_sort_and_list_to_del(
    quiet: bool,
    path: &path::Path,
    sort_type: &SortType,
    files_to_keep: u32,
    recursive: bool,
) -> io::Result<(Vec<path::PathBuf>, Vec<path::PathBuf>)> {
    if recursive {
        let all_groups = group_files_by_bucket_recursive(path, sort_type)?;
        let mut to_keep = Vec::new();
        let mut to_delete = Vec::new();
        for (dir, groups) in all_groups {
            let (keep, delete) =
                process_groups(quiet, &groups, sort_type, files_to_keep, &dir);
            to_keep.extend(keep);
            to_delete.extend(delete);
        }
        Ok((to_keep, to_delete))
    } else {
        let groups = group_files_by_bucket(path, sort_type)?;
        Ok(process_groups(quiet, &groups, sort_type, files_to_keep, path))
    }
}

fn delete_files(quiet: bool, files: &[path::PathBuf]) -> io::Result<()> {
    println_if_not_quiet!(quiet, "\nDeleting files...");
    for file in files {
        match fs::remove_file(file) {
            Ok(_) => println_if_not_quiet!(quiet, "File deleted: {}", file.display()),
            Err(e) => eprintln!("Error during deletion {}: {}", file.display(), e),
        }
    }
    Ok(())
}

fn process_groups(
    quiet: bool,
    groups: &collections::BTreeMap<u64, Vec<(path::PathBuf, time::SystemTime)>>,
    sort_type: &SortType,
    files_to_keep: u32,
    dir: &path::Path,
) -> (Vec<path::PathBuf>, Vec<path::PathBuf>) {
    let mut to_keep = Vec::new();
    let mut to_delete = Vec::new();
    println_if_not_quiet!(
        quiet,
        "\nOpening {}, sorting by {:?} and keeping {} files",
        dir.display(),
        sort_type,
        files_to_keep
    );
    for (bucket, files) in groups.iter() {
        println_if_not_quiet!(
            quiet,
            "\nYounger than {} days but older than {} days:",
            bucket,
            bucket / 2
        );
        let sorted: Vec<_> = files.iter().sorted_by_key(|(_, t)| *t).collect();
        let split_idx = files_to_keep.min(sorted.len() as u32) as usize;
        let (keep, delete) = sorted.split_at(split_idx);
        if delete.is_empty() {
            println_if_not_quiet!(quiet, "No files to delete in this group.");
        }
        for (file, time) in keep {
            let datetime: chrono::DateTime<chrono::Local> = (*time).into();
            println_if_not_quiet!(
                quiet,
                "{} | {}",
                file.display(),
                datetime.format("%Y-%m-%d %H:%M:%S")
            );
            to_keep.push(file.clone());
        }
        for (file, time) in delete {
            let datetime: chrono::DateTime<chrono::Local> = (*time).into();
            println_if_not_quiet!(
                quiet,
                "{} | {} <-- to be deleted",
                file.display(),
                datetime.format("%Y-%m-%d %H:%M:%S")
            );
            to_delete.push(file.clone());
        }
    }
    (to_keep, to_delete)
}

    // Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{FileTime, set_file_times};
    use gag::BufferRedirect;
    use rand::Rng;
    use std::io::Read;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
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
    fn test_listing_simple() {
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

        let result = exp_sort_and_list_to_del(
            false,
            dir.path(),
            &SortType::MTime,
            rng.random_range(1..5),
            false,
        );
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(
            false,
            dir.path(),
            &SortType::ATime,
            rng.random_range(1..5),
            false,
        );
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(
            false,
            dir.path(),
            &SortType::CTime,
            rng.random_range(1..5),
            false,
        ); //Can't modify ctime in tests so always one bucket
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
            exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 1, false).unwrap();

        assert!(to_keep.contains(&file1));
        assert!(to_delete.contains(&file3));
        assert!(to_delete.contains(&file4));
        assert!(to_delete.contains(&file2));
        assert_eq!(to_keep.len(), 1);
        assert_eq!(to_delete.len(), 3);

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(false, dir.path(), &SortType::ATime, 1, false).unwrap();
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

        thread::sleep(time::Duration::from_secs(2)); // Ensure a difference in ctime. That's why this test is slow.

        let file2 = dir.path().join("file2.txt");
        fs::File::create(&file2).unwrap();

        thread::sleep(time::Duration::from_secs(2));

        let file3 = dir.path().join("file3.txt");
        fs::File::create(&file3).unwrap();

        let (to_keep, to_delete) =
            exp_sort_and_list_to_del(false, dir.path(), &SortType::CTime, 1, false).unwrap();

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
            exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 1, false).unwrap();

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
            exp_sort_and_list_to_del(false, dir.path(), &SortType::ATime, 1, false).unwrap();

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
            exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 2, false).unwrap(); //Function deletes randomly. It is expected behavior for now. Maybe change in the future for asking the user.

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

        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 0, false);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::ATime, 0, false);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::CTime, 0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_directory() {
        println!("Testing with an empty directory");

        let dir = tempdir().unwrap();
        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 2, false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_invalid_path() {
        println!("Testing with an invalid path");

        let invalid_path = path::Path::new("/invalid/path");
        let result = exp_sort_and_list_to_del(false, invalid_path, &SortType::MTime, 2, false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_file_instead_of_directory() {
        println!("Testing with a file instead of a directory");

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        fs::File::create(&file_path).unwrap();
        let result = exp_sort_and_list_to_del(false, &file_path, &SortType::MTime, 2, false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotADirectory);
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

        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 1, false);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::ATime, 1, false);
        assert!(result.is_ok());
        let result = exp_sort_and_list_to_del(false, dir.path(), &SortType::CTime, 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn delete_files_test() {
        println!("Testing delete_files function");

        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        fs::File::create(&file1).unwrap();
        fs::File::create(&file2).unwrap();

        let files_to_delete = vec![file1.clone(), file2.clone()];
        let result = delete_files(false, &files_to_delete);
        assert!(result.is_ok());
        assert!(!file1.exists());
        assert!(!file2.exists());
    }

    #[test]
    fn delete_permission_denied() {
        println!("Testing delete_files function with permission denied scenario");

        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        fs::File::create(&file1).unwrap();

        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o555);
        fs::set_permissions(dir.path(), perms).unwrap();

        let files_to_delete = vec![file1.clone()];
        let result = delete_files(false, &files_to_delete);

        assert!(result.is_ok());
        assert!(file1.exists());
    }

    #[test]
    fn test_directory_with_subdirectories() {
        // Subdirectories should be ignored in non-recursive mode
        println!("Testing with a directory containing subdirectory");

        let dir = tempdir().unwrap();
        for i in 0..5 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            fs::File::create(&file_path).unwrap();
        }
        let sub_dir_path = dir.path().join("sub_dir");
        fs::create_dir(&sub_dir_path).unwrap();
        let subfile_path = sub_dir_path.join("subfile.txt");
        fs::File::create(&subfile_path).unwrap();

        let (_to_keep, to_delete) =
            exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 0, false).unwrap();
        delete_files(false, &to_delete).unwrap();

        assert!(dir.path().exists());
        for i in 0..5 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            assert!(!file_path.exists());
        }
        assert!(sub_dir_path.exists());
        assert!(subfile_path.exists());
    }

    #[test]
    fn test_directory_with_subdirectories_with_recursive_on() {
        // Subdirectories should NOT be ignored in recursive mode
        println!("Testing with a directory containing subdirectory with --recursive on");

        let dir = tempdir().unwrap();
        for i in 0..5 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            fs::File::create(&file_path).unwrap();
        }
        let sub_dir_path = dir.path().join("sub_dir");
        fs::create_dir(&sub_dir_path).unwrap();
        let subfile_path = sub_dir_path.join("subfile.txt");
        fs::File::create(&subfile_path).unwrap();

        let (_to_keep, to_delete) =
            exp_sort_and_list_to_del(false, dir.path(), &SortType::MTime, 0, true).unwrap();
        delete_files(false, &to_delete).unwrap();

        assert!(dir.path().exists());
        for i in 0..5 {
            let file_path = dir.path().join(format!("file{}.txt", i));
            assert!(!file_path.exists());
        }
        assert!(sub_dir_path.exists());
        assert!(!subfile_path.exists());
    }

    #[test]
    fn test_quiet_mode() {
        println!("Testing quiet mode");

        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        fs::File::create(&file1).unwrap();
        fs::File::create(&file2).unwrap();

        //Capture output
        let mut buf = Vec::new();
        let mut redirect = BufferRedirect::stdout().unwrap();

        let files_to_delete = vec![file1.clone(), file2.clone()];
        let result = delete_files(true, &files_to_delete);

        redirect.read_to_end(&mut buf).unwrap();
        assert!(
            buf.is_empty(),
            "Expected no output in quiet mode, but got some."
        );

        assert!(result.is_ok());
        assert!(!file1.exists());
        assert!(!file2.exists());
    }
}
