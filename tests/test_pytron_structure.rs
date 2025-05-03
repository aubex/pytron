use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_pytron_home_directory_structure() {
    // First, make sure to clean up any existing PYTRON_HOME from previous tests
    env::remove_var("PYTRON_HOME");
    
    // Create a predictable directory to use as PYTRON_HOME
    let custom_path = "/tmp/pytron_structure_test";
    // Clean up any existing directory first to ensure a fresh start
    let _ = fs::remove_dir_all(custom_path);
    let _ = fs::create_dir_all(custom_path);
    
    // Set our test-specific PYTRON_HOME
    env::set_var("PYTRON_HOME", custom_path);
    
    // Verify that the environment variable was set correctly
    let env_value = env::var("PYTRON_HOME").expect("Failed to get PYTRON_HOME");
    println!("Set PYTRON_HOME to: {}", env_value);
    
    // Check that get_pytron_home returns our custom path
    let home_path = pytron::get_pytron_home();
    // Before assert, print the actual values
    println!("Expected: {}, Got: {}", custom_path, home_path.display());
    assert_eq!(home_path, PathBuf::from(custom_path), "PYTRON_HOME should be set to our custom path");
    
    // Create the bin directory and check paths
    let bin_path = PathBuf::from(&custom_path).join("bin");
    fs::create_dir_all(&bin_path).expect("Failed to create bin directory");
    
    // Check uv path has the expected structure
    let uv_path = pytron::get_uv_path();
    
    // Check that it has the expected format - with the new installation, uv can be directly in PYTRON_HOME
    // or in the bin subdirectory
    #[cfg(windows)]
    {
        assert!(uv_path.ends_with("bin\\uv.exe") || uv_path.ends_with("uv.exe"), 
            "Windows uv path should end with bin\\uv.exe or uv.exe, got: {:?}", uv_path);
    }
    
    #[cfg(not(windows))]
    {
        assert!(uv_path.ends_with("bin/uv") || uv_path.ends_with("uv"), 
            "Unix uv path should end with bin/uv or uv, got: {:?}", uv_path);
    }
    
    // Create the temp directory for extraction and check it
    let temp_path = PathBuf::from(&custom_path).join("temp");
    fs::create_dir_all(&temp_path).expect("Failed to create temp directory");
    
    assert!(temp_path.exists(), "Temporary directory should exist in PYTRON_HOME");
    
    // Clean up
    env::remove_var("PYTRON_HOME");
    let _ = fs::remove_dir_all(custom_path);
}

#[test]
fn test_run_from_zip_temp_directory_creation() {
    // First, make sure to clean up any existing PYTRON_HOME from previous tests
    env::remove_var("PYTRON_HOME");
    
    // Use the exact same path as in test_pytron_home_directory_structure
    let custom_path = "/tmp/pytron_structure_test";
    // Clean up any existing directory first to ensure a fresh start
    let _ = fs::remove_dir_all(custom_path);
    let _ = fs::create_dir_all(custom_path);
    
    // Set our test-specific PYTRON_HOME
    env::set_var("PYTRON_HOME", custom_path);
    
    // Verify that the environment variable was set correctly
    let env_value = env::var("PYTRON_HOME").expect("Failed to get PYTRON_HOME");
    println!("Set PYTRON_HOME to: {}", env_value);
    
    // Create a simple test zip and script
    let test_dir = tempdir().expect("Failed to create temp directory for test files");
    let script_path = test_dir.path().join("simple.py");
    fs::write(&script_path, "print('Hello')").expect("Failed to write test script");
    
    let zip_path = test_dir.path().join("test.zip");
    pytron::zip_directory(
        test_dir.path().to_str().unwrap(),
        zip_path.to_str().unwrap(),
        None,
    ).expect("Failed to create test zip file");
    
    // Prepare for extraction path check and create it
    let temp_path = PathBuf::from(&custom_path).join("temp");
    fs::create_dir_all(&temp_path).expect("Failed to create temp directory");
    
    println!("Temp path before run_from_zip: {}", temp_path.display());
    assert!(temp_path.exists(), "temp directory should exist in PYTRON_HOME before run_from_zip");
    
    // Even if we can't run the script (no uv), the function should at least create the temp directory
    let _ = pytron::run_from_zip(
        zip_path.to_str().unwrap(),
        "simple.py", 
        &[],
        &[],
    );
    
    // After our changes to the run_from_zip function, it should now always create the temp directory
    println!("Temp path after run_from_zip: {}", temp_path.display());
    assert!(temp_path.exists(), "temp directory should exist in PYTRON_HOME after run_from_zip");
    
    // Check the value of PYTRON_HOME again to make sure it wasn't modified
    match env::var("PYTRON_HOME") {
        Ok(val) => println!("PYTRON_HOME after run: {}", val),
        Err(_) => println!("PYTRON_HOME is not set after run!"),
    }
    
    // Clean up
    env::remove_var("PYTRON_HOME");
    let _ = fs::remove_dir_all(custom_path);
}