use pytron::{delete_python_cache ,show_cache_dirs};
use tempfile::tempdir;
use std::fs::{self, File, create_dir_all};
use std::path::Path;

fn create_file<P: AsRef<Path>>(path: P) {
    File::create(path).expect("Failed to create file");
}

fn create_dir_structure(base: &Path, names: &[&str]) {
    for name in names {
        let dir = base.join(name);
        fs::create_dir_all(&dir).unwrap();
        create_file(dir.join("dummy.pyc")); // dummy cache file
    }
}

#[test]
fn test_delete_python_cache_basic() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    create_dir_structure(root, &["__pycache__", ".pytest_cache", ".venv", "not_cache"]);

    assert!(root.join("__pycache__").exists());
    assert!(root.join(".pytest_cache").exists());
    assert!(root.join(".venv").exists());

    delete_python_cache(root).unwrap();

    // Check deletion
    assert!(!root.join("__pycache__").exists());
    assert!(!root.join(".pytest_cache").exists());
    assert!(!root.join(".venv").exists());

    // Check non-cache folder still exists
    assert!(root.join("not_cache").exists());
}

#[test]
fn test_delete_nested_cache_dirs() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let subdir = root.join("src/module");
    create_dir_all(&subdir).unwrap();
    create_dir_structure(&subdir, &["__pycache__", ".pytest_cache"]);

    delete_python_cache(root).unwrap();

    assert!(!subdir.join("__pycache__").exists());
    assert!(!subdir.join(".pytest_cache").exists());
}

#[test]
fn test_show_cache_dirs() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    create_dir_structure(root, &["__pycache__", ".pytest_cache", ".venv"]);

    // Note: Here we just want to ensure it runs. Output would go to stdout.
    assert_eq!(show_cache_dirs(root).unwrap(), 0);
}

#[test]
fn test_delete_python_cache_no_cache_dirs() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    create_dir_structure(root, &["src", "docs"]);

    assert_eq!(delete_python_cache(root).unwrap(), 0);
    assert!(root.join("src").exists());
    assert!(root.join("docs").exists());
}

#[test]
fn test_ignore_files_and_non_dirs() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let file_path = root.join("__pycache__"); // should be a file, not a dir
    File::create(&file_path).unwrap();

    assert!(file_path.is_file());
    assert_eq!(delete_python_cache(root).unwrap(), 0);
    assert!(file_path.exists()); // should not be deleted
}