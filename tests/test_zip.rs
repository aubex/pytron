use pytron::zip_directory;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

// Helper function to create a test directory with files
fn create_test_directory() -> tempfile::TempDir {
    let dir = tempdir().expect("Failed to create temp directory");

    // Create a simple Python script
    let script_path = dir.path().join("main.py");
    let mut script_file = File::create(&script_path).expect("Failed to create main.py");
    script_file
        .write_all(b"print('Hello from test!')\n")
        .expect("Failed to write to main.py");

    // Create a subdirectory with another file
    let subdir_path = dir.path().join("subdir");
    fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

    let subfile_path = subdir_path.join("helper.py");
    let mut subfile = File::create(&subfile_path).expect("Failed to create helper.py");
    subfile
        .write_all(b"def helper():\n    return 'Helper function'\n")
        .expect("Failed to write to helper.py");

    // Create a .gitignore file
    let gitignore_path = dir.path().join(".gitignore");
    let mut gitignore_file = File::create(&gitignore_path).expect("Failed to create .gitignore");
    gitignore_file
        .write_all(b"*.log\n*.tmp\n")
        .expect("Failed to write to .gitignore");

    // Create files that should be ignored
    let ignored_file = dir.path().join("ignored.log");
    let mut ignored = File::create(&ignored_file).expect("Failed to create ignored.log");
    ignored
        .write_all(b"This file should be ignored\n")
        .expect("Failed to write to ignored.log");

    dir
}

// Test zip_directory function
#[test]
fn test_zip_directory() {
    let test_dir = create_test_directory();
    let output_zip = test_dir.path().join("test_output.zip");

    // Call the zip_directory function with default ignore patterns
    let result = zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        None,
    );

    // Verify the function succeeded
    assert!(result.is_ok(), "zip_directory failed: {:?}", result.err());

    // Verify the zip file was created
    assert!(output_zip.exists(), "Zip file was not created");

    // Open and verify the zip contents
    let file = File::open(&output_zip).expect("Failed to open zip file");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip archive");

    // Check file count (should be 3 files: main.py, .gitignore, subdir/helper.py)
    assert_eq!(archive.len(), 3, "Zip archive should contain 3 files");

    // Verify specific files are present
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| {
            let name = archive.by_index(i).unwrap().name().to_string();
            // Normalize backslashes to forward slashes for cross-platform compatibility
            name.replace('\\', "/")
        })
        .collect();

    assert!(
        file_names.contains(&"main.py".to_string()),
        "main.py is missing from the archive"
    );
    assert!(
        file_names.contains(&".gitignore".to_string()),
        ".gitignore is missing from the archive"
    );
    assert!(
        file_names.contains(&"subdir/helper.py".to_string()),
        "subdir/helper.py is missing from the archive"
    );

    // Print all files for debugging
    println!("Files in archive: {:?}", file_names);

    // Verify ignored files are not present
    assert!(
        !file_names.contains(&"ignored.log".to_string()),
        "ignored.log should not be in the archive"
    );
}

#[test]
fn test_zip_directory_with_custom_ignore() {
    let test_dir = create_test_directory();
    
    // Create additional test files
    let subdir_path = test_dir.path().join("subdir");
    let custom_ignore_file = subdir_path.join("custom_ignore.txt");
    let mut custom_file = File::create(&custom_ignore_file).expect("Failed to create custom_ignore.txt");
    custom_file
        .write_all(b"This file should be ignored by custom pattern\n")
        .expect("Failed to write to custom_ignore.txt");
    
    let output_zip = test_dir.path().join("custom_ignore_output.zip");
    
    // Custom ignore patterns that will ignore any file with "custom_ignore" in the name
    let custom_patterns = Some(vec!["*custom_ignore*".to_string()]);
    
    // Call the zip_directory function with custom ignore patterns
    let result = zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        custom_patterns.as_ref(),
    );
    
    // Verify the function succeeded
    assert!(result.is_ok(), "zip_directory with custom patterns failed: {:?}", result.err());
    
    // Open and verify the zip contents
    let file = File::open(&output_zip).expect("Failed to open zip file");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip archive");
    
    // Get all file names in the archive and normalize path separators
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| {
            let name = archive.by_index(i).unwrap().name().to_string();
            // Normalize backslashes to forward slashes for cross-platform compatibility
            name.replace('\\', "/")
        })
        .collect();
    
    println!("Files in custom ignore archive: {:?}", file_names);
    
    // Verify the custom-ignored file is not present
    assert!(
        !file_names.contains(&"subdir/custom_ignore.txt".to_string()),
        "custom_ignore.txt should not be in the archive"
    );
    
    // Verify standard ignored files are also not present
    assert!(
        !file_names.contains(&"ignored.log".to_string()),
        "ignored.log should not be in the archive"
    );
}

#[test]
fn test_zip_directory_override_defaults() {
    let test_dir = create_test_directory();
    
    // Create a .git directory that would normally be ignored
    let git_dir = test_dir.path().join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git directory");
    let git_file = git_dir.join("HEAD");
    let mut git_head = File::create(&git_file).expect("Failed to create .git/HEAD");
    git_head
        .write_all(b"ref: refs/heads/main\n")
        .expect("Failed to write to .git/HEAD");
    
    let output_zip = test_dir.path().join("override_defaults_output.zip");
    
    // Empty string to override default excludes
    let override_patterns = Some(vec!["".to_string()]);
    
    // Call the zip_directory function with overridden default patterns
    let result = zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        override_patterns.as_ref(),
    );
    
    // Verify the function succeeded
    assert!(result.is_ok(), "zip_directory with overridden defaults failed: {:?}", result.err());
    
    // Open and verify the zip contents
    let file = File::open(&output_zip).expect("Failed to open zip file");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip archive");
    
    // Get all file names in the archive and normalize path separators
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| {
            let name = archive.by_index(i).unwrap().name().to_string();
            // Normalize backslashes to forward slashes for cross-platform compatibility
            name.replace('\\', "/")
        })
        .collect();
    
    println!("Files in override defaults archive: {:?}", file_names);
    
    // Verify the .git directory contents are now included (since default excludes are overridden)
    // But .gitignore rules still apply, so .log files should still be excluded
    assert!(
        file_names.contains(&".git/HEAD".to_string()),
        ".git/HEAD should be in the archive since default excludes are overridden"
    );
    
    // Files specified in .gitignore should still be excluded
    assert!(
        !file_names.contains(&"ignored.log".to_string()),
        "ignored.log should still not be in the archive (from .gitignore)"
    );
}