use std::fs::{self, File};
use std::io::Write;
use tempfile::{tempdir, TempDir};

// Helper function to create a test directory with files
#[cfg(test)]
pub fn create_test_directory() -> TempDir {
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

// Helper function to create test directory with scripts that return different exit codes
#[cfg(test)]
pub fn create_exit_code_test_directory() -> TempDir {
    let dir = tempdir().expect("Failed to create temp directory");

    // Copy the test files from fixtures to the temp directory
    let fixtures = [
        "test_exit_code_0.py",
        "test_exit_code_1.py",
        "test_exit_code_custom.py",
    ];

    for fixture in fixtures {
        let fixture_path = format!("tests/fixtures/{}", fixture);
        let dest_path = dir.path().join(fixture);

        // Read the fixture content
        let content = std::fs::read_to_string(&fixture_path)
            .expect(&format!("Failed to read fixture {}", fixture_path));

        // Write it to the temp directory
        let mut file =
            File::create(&dest_path).expect(&format!("Failed to create {}", dest_path.display()));
        file.write_all(content.as_bytes())
            .expect(&format!("Failed to write to {}", dest_path.display()));

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = dest_path.metadata().unwrap();
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest_path, perms).unwrap();
        }
    }

    dir
}
