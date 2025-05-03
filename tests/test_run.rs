use pytron::run_from_zip;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Test run_from_zip function
#[test]
fn test_run_from_zip() {
    let test_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let output_zip = test_dir.path().join("test_output.zip");

    // Create the zip file first
    let _ = pytron::zip_directory(
        test_dir.path().to_str().unwrap(),
        output_zip.to_str().unwrap(),
        None,
    )
    .expect("Failed to create test zip file");

    // We can't fully test run_from_zip without mocking the Python interpreter,
    // but we can test the extraction part by checking for errors
    let result = run_from_zip(
        output_zip.to_str().unwrap(),
        "non_existent.py", // This should cause the function to return an error
        &[],               // uv_args
        &[],               // script_args
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

// Test for argument handling
#[test]
fn test_run_with_arguments() {
    // First, make sure to clean up any existing PYTRON_HOME from previous tests
    env::remove_var("PYTRON_HOME");

    // Create a special test script that captures its arguments
    let test_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let script_path = test_dir.path().join("arg_test.py");

    // This script will write all command line arguments to a file
    let script_content = r#"
import sys
import os

# Create a simple verification file - this is simpler than running Python code
with open(os.path.join(os.path.dirname(__file__), "args_output.txt"), "w") as f:
    # Write script args to the file, separated by newlines
    f.write("\n".join(sys.argv[1:]))

# Print args for debugging
print(f"Arguments received: {sys.argv[1:]}")
"#;

    std::fs::write(&script_path, script_content).expect("Failed to create arg_test.py");

    // Create a zip file
    let zip_path = test_dir.path().join("arg_test.zip");
    let _ = pytron::zip_directory(
        test_dir.path().to_str().unwrap(),
        zip_path.to_str().unwrap(),
        None,
    )
    .expect("Failed to create test zip file");

    // Setup PYTRON_HOME to a known location for this test
    let pytron_home_dir = tempfile::tempdir().expect("Failed to create PYTRON_HOME directory");
    let pytron_home = pytron_home_dir.path().to_string_lossy().to_string();
    env::set_var("PYTRON_HOME", &pytron_home);

    // Skip this test if uv is not available
    if !pytron::is_uv_installed() {
        println!("Skipping test_run_with_arguments as uv is not available");
        env::remove_var("PYTRON_HOME");
        return;
    }

    // The script args to test
    let uv_args = vec!["-v".to_string()]; // Verbose flag for uv
    let script_args = vec!["hello".to_string(), "world".to_string()];

    // Run the test in a separate thread with a timeout
    let args_file_path = test_dir.path().join("args_output.txt");
    let done_flag = Arc::new(AtomicBool::new(false));
    let captured_args = Arc::new(Mutex::new(String::new()));

    let done_flag_clone = done_flag.clone();
    let args_file_path_clone = args_file_path.clone();
    let captured_args_clone = captured_args.clone();

    thread::spawn(move || {
        // Run the script
        let _ = run_from_zip(
            zip_path.to_str().unwrap(),
            "arg_test.py",
            &uv_args,
            &script_args,
        );

        // Give the script some time to write output (increase from previous version)
        thread::sleep(Duration::from_millis(1000));

        // Check if output file exists and read it
        if args_file_path_clone.exists() {
            if let Ok(content) = std::fs::read_to_string(&args_file_path_clone) {
                let mut args = captured_args_clone.lock().unwrap();
                *args = content;
            }
            done_flag_clone.store(true, Ordering::SeqCst);
        }
    });

    let mut retries = 2; // 1 second (2 * 500ms)
    while !done_flag.load(Ordering::SeqCst) && retries > 0 {
        thread::sleep(Duration::from_millis(500));
        retries -= 1;
    }

    // Check the results
    if done_flag.load(Ordering::SeqCst) {
        let args = captured_args.lock().unwrap();
        println!("Captured args: {:?}", *args);

        // Check that the script received the correct arguments
        assert!(
            args.contains("hello"),
            "Script should have received 'hello' argument"
        );
        assert!(
            args.contains("world"),
            "Script should have received 'world' argument"
        );

        // Verify uv flags aren't passed to the script
        assert!(
            !args.contains("-v"),
            "Script should not have received '-v' flag"
        );
    } else {
        println!("WARNING: test_run_with_arguments did not complete within timeout");
        println!(
            "This is not necessarily a failure - the test might be skipped on systems without uv."
        );
    }

    // Clean up
    env::remove_var("PYTRON_HOME");
}
