use clap::Parser;
use std::fs::{self, File};
use std::io::Write;
use tempfile::{tempdir, TempDir};

// Helper function to create a test directory with files
fn create_test_directory() -> TempDir {
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
    let result = super::zip_directory(
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
        .map(|i| archive.by_index(i).unwrap().name().to_string())
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
    let result = super::zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        custom_patterns.as_ref(),
    );
    
    // Verify the function succeeded
    assert!(result.is_ok(), "zip_directory with custom patterns failed: {:?}", result.err());
    
    // Open and verify the zip contents
    let file = File::open(&output_zip).expect("Failed to open zip file");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip archive");
    
    // Get all file names in the archive
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
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
    let result = super::zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        override_patterns.as_ref(),
    );
    
    // Verify the function succeeded
    assert!(result.is_ok(), "zip_directory with overridden defaults failed: {:?}", result.err());
    
    // Open and verify the zip contents
    let file = File::open(&output_zip).expect("Failed to open zip file");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip archive");
    
    // Get all file names in the archive
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
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

// Test run_from_zip function
#[test]
fn test_run_from_zip() {
    let test_dir = create_test_directory();
    let output_zip = test_dir.path().join("test_output.zip");

    // Create the zip file first
    let _ = super::zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        None,
    )
    .expect("Failed to create test zip file");

    // We can't fully test run_from_zip without mocking the Python interpreter,
    // but we can test the extraction part by checking for errors
    let result = super::run_from_zip(
        output_zip.to_str().unwrap(),
        "non_existent.py", // This should cause the function to return an error
        &[],
    );

    // Verify we get the expected error for a non-existent script
    assert!(
        result.is_err(),
        "Expected an error when running non-existent script"
    );
    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("not found"),
        "Expected 'not found' error, got: {}",
        error
    );
}

// Test CLI argument parsing
#[test]
fn test_cli_parsing() {
    // Test the Zip command with defaults
    let args = vec!["pytron", "zip"];
    let cli = super::Cli::parse_from(args);

    if let super::Commands::Zip { directory, output, ignore_patterns } = cli.command {
        assert_eq!(directory, ".", "Default directory should be '.'");
        assert_eq!(
            output, "robot.zip",
            "Default output file should be 'robot.zip'"
        );
        assert!(ignore_patterns.is_none(), "Default ignore_patterns should be None");
    } else {
        panic!("Expected Zip command");
    }

    // Test the Zip command with custom ignore patterns
    let args = vec!["pytron", "zip", "--ignore-patterns", "node_modules,*.log,*.tmp"];
    let cli = super::Cli::parse_from(args);

    if let super::Commands::Zip { directory, output, ignore_patterns } = cli.command {
        assert_eq!(directory, ".", "Default directory should be '.'");
        assert_eq!(
            output, "robot.zip",
            "Default output file should be 'robot.zip'"
        );
        assert!(ignore_patterns.is_some(), "Custom ignore_patterns should be Some");
        let patterns = ignore_patterns.unwrap();
        assert_eq!(patterns.len(), 3, "Expected 3 ignore patterns");
        assert_eq!(patterns[0], "node_modules", "First pattern should be 'node_modules'");
        assert_eq!(patterns[1], "*.log", "Second pattern should be '*.log'");
        assert_eq!(patterns[2], "*.tmp", "Third pattern should be '*.tmp'");
    } else {
        panic!("Expected Zip command");
    }

    // Test the Zip command with empty string to override defaults
    let args = vec!["pytron", "zip", "--ignore-patterns", ""];
    let cli = super::Cli::parse_from(args);

    if let super::Commands::Zip { directory, output, ignore_patterns } = cli.command {
        assert_eq!(directory, ".", "Default directory should be '.'");
        assert_eq!(
            output, "robot.zip",
            "Default output file should be 'robot.zip'"
        );
        assert!(ignore_patterns.is_some(), "Empty ignore_patterns should be Some");
        let patterns = ignore_patterns.unwrap();
        assert_eq!(patterns.len(), 1, "Expected 1 empty string");
        assert_eq!(patterns[0], "", "Pattern should be empty string");
    } else {
        panic!("Expected Zip command");
    }

    // Test the Run command with defaults
    let args = vec!["pytron", "run"];
    let cli = super::Cli::parse_from(args);

    if let super::Commands::Run {
        zipfile,
        script,
        args,
    } = cli.command
    {
        assert_eq!(
            zipfile, "robot.zip",
            "Default zip file should be 'robot.zip'"
        );
        assert_eq!(script, "main.py", "Default script should be 'main.py'");
        assert_eq!(args.len(), 0, "No script arguments expected");
    } else {
        panic!("Expected Run command");
    }

    // Test the Run command with custom values
    let args = vec!["pytron", "run", "custom.zip", "custom.py", "arg1", "arg2"];
    let cli = super::Cli::parse_from(args);

    if let super::Commands::Run {
        zipfile,
        script,
        args,
    } = cli.command
    {
        assert_eq!(zipfile, "custom.zip", "Custom zip file name not matched");
        assert_eq!(script, "custom.py", "Custom script name not matched");
        assert_eq!(args.len(), 2, "Expected 2 script arguments");
        assert_eq!(args[0], "arg1", "First argument should be 'arg1'");
        assert_eq!(args[1], "arg2", "Second argument should be 'arg2'");
    } else {
        panic!("Expected Run command");
    }
}
