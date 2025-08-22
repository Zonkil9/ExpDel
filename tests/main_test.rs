use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::tempdir;
use std::{fs, time};
use filetime::{set_file_times, FileTime};
use rand::Rng;

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
    let input = format!(
        "{}\nmtime\n2\nyes\n",
        dir.path().display()
    );

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    println!("\n Provided input:\n{}", input);
    cmd.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    let output = cmd.wait_with_output().unwrap();
    println!("Program output: {}", String::from_utf8_lossy(&output.stdout));

    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert!(remaining_files <= 20); // 10 time segments, max 2 files per segment
    dir.close().unwrap();
}

#[test]
fn test_main_integration_ctime() {
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
    let input = format!(
        "{}\nctime\n3\nyes\n",
        dir.path().display()
    );

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    println!("\n Provided input:\n{}", input);
    cmd.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    let output = cmd.wait_with_output().unwrap();
    println!("Program output: {}", String::from_utf8_lossy(&output.stdout));

    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert!(remaining_files == 3); // Always 3 files should remain, because ctime can't be changed
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
    let input = format!(
        "{}\natime\n4\nyes\n",
        dir.path().display()
    );

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ExpDel"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    println!("\n Provided input:\n{}", input);
    cmd.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    let output = cmd.wait_with_output().unwrap();
    println!("Program output: {}", String::from_utf8_lossy(&output.stdout));

    assert!(output.status.success());

    // Check that files are deleted
    let remaining_files = fs::read_dir(dir.path()).unwrap().count();
    println!("\nRemaining files: {}", remaining_files);
    assert!(remaining_files <= 40); // 10 time segments, max 4 files per segment
    dir.close().unwrap();
}