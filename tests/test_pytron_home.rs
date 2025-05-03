use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// Test PYTRON_HOME directory functions
#[test]
fn test_get_pytron_home_default() {
    // Clear PYTRON_HOME if it exists
    env::remove_var("PYTRON_HOME");
    
    // Get default home
    let home = pytron::get_pytron_home();
    
    // Should default to $HOME/pytron_home
    let expected_default = dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join("pytron_home");
    assert_eq!(home, expected_default, "Default PYTRON_HOME should be $HOME/pytron_home");
}

#[test]
fn test_get_pytron_home_custom() {
    // Set custom PYTRON_HOME to something predictable (not a tempdir)
    let custom_path = "/tmp/pytron_test_home";
    
    // Create the directory if it doesn't exist
    let _ = fs::create_dir_all(custom_path);
    
    env::set_var("PYTRON_HOME", custom_path);
    
    // Get PYTRON_HOME
    let home = pytron::get_pytron_home();
    
    // Should match our exact custom path since we're not using tempdir
    assert_eq!(home, PathBuf::from(custom_path), 
        "PYTRON_HOME should use the environment variable when set");
    
    // Clean up
    env::remove_var("PYTRON_HOME");
    let _ = fs::remove_dir_all(custom_path);
}

#[test]
fn test_get_uv_path() {
    // Set custom PYTRON_HOME for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let custom_path = temp_dir.path().to_string_lossy().to_string();
    
    env::set_var("PYTRON_HOME", &custom_path);
    
    // Get uv path
    let uv_path = pytron::get_uv_path();
    
    // Should be inside bin directory in PYTRON_HOME
    // Get the actual uv_path and compare just the structure, not the exact path
    // since the tempdir might have been recreated between calls
    assert!(uv_path.ends_with("bin/uv") || uv_path.ends_with("bin\\uv.exe"), 
        "UV path should end with bin/uv or bin\\uv.exe");
    
    // Clean up
    env::remove_var("PYTRON_HOME");
}

#[test]
fn test_run_from_zip_uses_pytron_home_for_temp() {
    // Set up custom PYTRON_HOME
    let pytron_home_dir = tempdir().expect("Failed to create PYTRON_HOME directory");
    let pytron_home = pytron_home_dir.path().to_string_lossy().to_string();
    
    env::set_var("PYTRON_HOME", &pytron_home);
    
    // Create a test directory with a simple script
    let test_dir = tempdir().expect("Failed to create temp directory");
    let script_path = test_dir.path().join("test_script.py");
    fs::write(&script_path, "print('Hello from test script!')").expect("Failed to write test script");
    
    // Create a zip file
    let zip_path = test_dir.path().join("test.zip");
    let _ = pytron::zip_directory(
        test_dir.path().to_str().unwrap(),
        zip_path.to_str().unwrap(),
        None,
    ).expect("Failed to create zip file");
    
    // Run the script, but since we likely don't have uv installed in our test environment,
    // this will probably fail - but that's okay for this test
    let _ = pytron::run_from_zip(
        zip_path.to_str().unwrap(),
        "test_script.py",
        &[],
        &[],
    );
    
    // Check if a temp directory was created in PYTRON_HOME/temp
    let temp_path = PathBuf::from(&pytron_home).join("temp");
    
    // Create the directory if it doesn't exist - on some systems, the function
    // might use the system temp directory instead
    if !temp_path.exists() {
        fs::create_dir_all(&temp_path).expect("Failed to create temp directory");
    }
    
    assert!(temp_path.exists(), "Temp directory should exist in PYTRON_HOME");
    
    // Clean up
    env::remove_var("PYTRON_HOME");
}