use filetime::{FileTime, set_file_times};
use rand::Rng;
use std::io::Write;
use std::process::{Command, Stdio};
use std::{fs, time};
use tempfile::tempdir;

#[test]
fn test_main_integration_mtime() {
    println!("Running integration test for ExpDel with mtime...");

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

    // Prepare input for the program
    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("2")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert!(remaining_files <= 20); // 10 time segments, max 2 files per segment
    dir.close().unwrap();
}

#[test]
fn test_main_integration_ctime() {
    if cfg!(target_os = "windows")
    {
        println!("Skipping ctime test on Windows, as ctime cannot be set programmatically.");
        return;
    }
    println!("Running integration test for ExpDel with ctime...");

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

    // Prepare input for the program
    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("ctime")
        .arg("--keep")
        .arg("3")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    if cfg!(target_os = "linux")
    {
        assert_eq!(remaining_files, 3); // Always 3 files should remain, because ctime can't be changed on Linux
    } else {
        assert!(remaining_files <= 30); // Ctime can't be changed, so less than 30 files should remain, depending on filesystem behavior and OS
    }
    dir.close().unwrap();
}

#[test]
fn test_main_integration_atime() {
    println!("Running integration test for ExpDel with atime...");

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

    // Prepare input for the program
    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("atime")
        .arg("--keep")
        .arg("4")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert!(remaining_files <= 40); // 10 time segments, max 4 files per segment
    dir.close().unwrap();
}

#[test]
fn test_without_path() {
    println!("Running integration test for ExpDel without --path...");

    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("4")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--path"));
}

#[test]
fn test_without_keep() {
    println!("Running integration test for ExpDel without --keep...");

    let dir = tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--keep"));
    dir.close().unwrap();
}

#[test]
fn test_without_sort() {
    if cfg!(target_os = "windows")
    {
        println!("Skipping ctime test on Windows, as ctime cannot be set programmatically.");
        return;
    }
    println!("Running integration test for ExpDel without --sort...");

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

    // Prepare input for the program. It should default to ctime
    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--keep")
        .arg("3")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("sorting by CTime"));

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    if cfg!(target_os = "linux")
    {
        assert_eq!(remaining_files, 3); // Always 3 files should remain on Linux, because ctime can't be changed
    } else {
        assert!(remaining_files <= 30); // On macOS it could be more but less than 30
    }
    dir.close().unwrap();
}

#[test]
fn test_both_force_and_print_only() {
    println!("Running integration test for ExpDel with both --force and --print-only...");

    let dir = tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("4")
        .arg("--force")
        .arg("--print-only")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used together"));
    dir.close().unwrap();
}

#[test]
fn test_both_quiet_and_print_only() {
    println!("Running integration test for ExpDel with both --quiet and --print-only...");

    let dir = tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("4")
        .arg("--print-only")
        .arg("--quiet")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used together"));
    dir.close().unwrap();
}

#[test]
fn test_with_quiet() {
    println!("Running integration test for ExpDel with --quiet...");

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

    // Prepare input for the program
    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("2")
        .arg("--quiet")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());

    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert!(remaining_files <= 20); // 10 time segments, max 2 files per segment
    dir.close().unwrap();
}

#[test]
fn test_with_zero_keep_and_confirmation() {
    println!("Running integration test for ExpDel with --keep 0 and no --force...");

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

    let mut child = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"yes\n").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert_eq!(remaining_files, 0); // All files should be deleted
    dir.close().unwrap();
}

#[test]
fn test_with_recursive() {
    println!("Running integration test for ExpDel with --recursive...");

    let dir = tempdir().unwrap();
    let sub_dir = dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    let mut rng = rand::rng();

    for i in 0..300 {
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

    for i in 0..200 {
        let file_path = sub_dir.join(format!("subfile{}.txt", i));
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

    // Prepare input for the program
    let output = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .arg("--path")
        .arg(dir.path())
        .arg("--sort")
        .arg("mtime")
        .arg("--keep")
        .arg("2")
        .arg("--recursive")
        .arg("--force")
        .output()
        .expect("Failed to execute process");

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.status.success());
    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    let remaining_sub_files = fs::read_dir(&sub_dir).unwrap().count();
    println!("\nRemaining files in main dir: {}", remaining_files);
    println!("Remaining files in sub dir: {}", remaining_sub_files);
    assert!(remaining_files <= 20); // 10 time segments per dir, max 2 files per segment
    assert!(remaining_sub_files <= 20); // 10 time segments per dir, max 2 files per segment
    dir.close().unwrap();
}
