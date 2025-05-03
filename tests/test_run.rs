use pytron::run_from_zip;

mod common;

// Test run_from_zip function
#[test]
fn test_run_from_zip() {
    let test_dir = common::create_test_directory();
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
