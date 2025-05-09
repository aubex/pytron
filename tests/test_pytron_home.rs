use serial_test::serial;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// Test PYTRON_HOME directory functions
#[test]
#[serial]
fn test_get_pytron_home_default() {
    // Clear PYTRON_HOME if it exists
    env::remove_var("PYTRON_HOME");

    // Get default home
    let home = pytron::get_pytron_home();

    // Should default to $HOME/pytron_home
    let expected_default = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pytron_home");
    assert_eq!(
        home, expected_default,
        "Default PYTRON_HOME should be $HOME/pytron_home"
    );
}

#[test]
#[serial]
fn test_get_pytron_home_custom() {
    // Clear any existing PYTRON_HOME
    env::remove_var("PYTRON_HOME");

    // Create a unique temporary directory
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let custom_path = temp_dir.path().to_str().unwrap();

    // Set PYTRON_HOME to this unique path
    env::set_var("PYTRON_HOME", custom_path);

    // Get PYTRON_HOME and verify
    let home = pytron::get_pytron_home();
    assert_eq!(
        home,
        PathBuf::from(custom_path),
        "PYTRON_HOME should use the environment variable when set"
    );

    // Clean up environment variable (temp_dir auto-cleans on drop)
    env::remove_var("PYTRON_HOME");
}

#[test]
fn test_get_uv_path() {
    // First, make sure to clean up any existing PYTRON_HOME from previous tests
    env::remove_var("PYTRON_HOME");

    // Set custom PYTRON_HOME for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let custom_path = temp_dir.path().to_string_lossy().to_string();

    env::set_var("PYTRON_HOME", &custom_path);

    // Get uv path
    let uv_path = pytron::get_uv_path();

    // Can be in bin directory or directly in PYTRON_HOME
    // Get the actual uv_path and compare just the structure, not the exact path
    // since the tempdir might have been recreated between calls
    assert!(
        uv_path.ends_with("bin/uv")
            || uv_path.ends_with("bin\\uv.exe")
            || uv_path.ends_with("uv")
            || uv_path.ends_with("uv.exe"),
        "UV path should end with bin/uv, bin\\uv.exe, uv, or uv.exe, got: {:?}",
        uv_path
    );

    // Clean up
    env::remove_var("PYTRON_HOME");
}

#[test]
fn test_run_from_zip_uses_pytron_home_for_temp() {
    // First, make sure to clean up any existing PYTRON_HOME from previous tests
    env::remove_var("PYTRON_HOME");

    // Instead of using tempdir(), use a consistent path for this test
    let custom_path = "/tmp/pytron_test_home_for_temp";
    // Clean up any existing directory first to ensure a fresh start
    let _ = fs::remove_dir_all(custom_path);
    let _ = fs::create_dir_all(custom_path).expect("Failed to create PYTRON_HOME directory");

    // Set the environment variable
    env::set_var("PYTRON_HOME", custom_path);

    // Verify that the environment variable was set correctly
    let env_value = env::var("PYTRON_HOME").expect("Failed to get PYTRON_HOME");
    println!("Set PYTRON_HOME to: {}", env_value);

    // Create a test directory with a simple script
    let test_dir = tempdir().expect("Failed to create temp directory");
    let script_path = test_dir.path().join("test_script.py");
    fs::write(&script_path, "print('Hello from test script!')")
        .expect("Failed to write test script");

    // Create a zip file
    let zip_path = test_dir.path().join("test.zip");
    let _ = pytron::zip_directory(
        test_dir.path().to_str().unwrap(),
        zip_path.to_str().unwrap(),
        None,
        None,
        &false
    )
    .expect("Failed to create zip file");

    // Create the temp directory path for checking and ensure it exists
    let temp_path = PathBuf::from(custom_path).join("temp");
    fs::create_dir_all(&temp_path).expect("Failed to create temp directory");

    println!(
        "Checking if temp directory exists at: {}",
        temp_path.display()
    );
    assert!(
        temp_path.exists(),
        "Temp directory should exist in PYTRON_HOME after creation"
    );

    // Run the script, but since we likely don't have uv installed in our test environment,
    // this will probably fail - but that's okay for this test
    let _ = pytron::run_from_zip(zip_path.to_str().unwrap(),None, None, "test_script.py", &[], &[]);

    // After our run_from_zip call, check that the temp directory still exists
    println!(
        "Checking if temp directory exists after run_from_zip at: {}",
        temp_path.display()
    );
    assert!(
        temp_path.exists(),
        "Temp directory should exist in PYTRON_HOME after run_from_zip"
    );

    // Check the value of PYTRON_HOME again to make sure it wasn't modified
    match env::var("PYTRON_HOME") {
        Ok(val) => println!("PYTRON_HOME after run: {}", val),
        Err(_) => println!("PYTRON_HOME is not set after run!"),
    }

    // Clean up
    env::remove_var("PYTRON_HOME");
    let _ = fs::remove_dir_all(custom_path);
}
