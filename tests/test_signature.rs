use pytron::signature::{sign_zip, verify_zip};
use tempfile::tempdir;
use std::{fs, io::Write, path::PathBuf};
use ed25519_dalek::{Keypair, Signer, PUBLIC_KEY_LENGTH};
use rand::rngs::OsRng;


const MARKER: [u8; 4] = [0x05, 0x04, 0x07, 0x07];

/// Helper to read all bytes from a file.
fn read_bytes(path: &PathBuf) -> Vec<u8> {
    fs::read(path).expect("failed to read file")
}

#[test]
fn test_sign_zip_success() {
    // Create temp dir and a fake ZIP file.
    let dir = tempdir().expect("failed to create tempdir");
    let zip_path = dir.path().join("test.zip");
    let mut file = fs::File::create(&zip_path).expect("create zip");
    file.write_all(b"dummy-zip-content").expect("write");

    let result = sign_zip(zip_path.to_str().unwrap());
    assert!(result.is_ok(), "Expected signing to succeed, got {:?}", result);

    let signed = read_bytes(&zip_path);

    // ZIP file should end with the 4-byte marker + 64-byte signature.
    assert!(
        signed.len() >= 68,
        "Signed file too small: {} bytes",
        signed.len()
    );
    let (_body, tail) = signed.split_at(signed.len() - (68));
    let (marker_bytes, signature_bytes) = tail.split_at(4);

    assert_eq!(marker_bytes, MARKER, "Marker not found at end");
    assert_eq!(
        signature_bytes.len(),
        64,
        "Signature is not 64 bytes"
    );

    // Check that .key file exists and is 32 bytes (ed25519 public key).
    let key_path = dir.path().join("test.key");
    let pubkey = read_bytes(&key_path);
    assert_eq!(pubkey.len(), 32, "Public key should be 32 bytes");
}

#[test]
fn test_sign_zip_already_signed_error() {
    // Create a temp ZIP file that already has the marker
    let dir = tempdir().expect("failed to create tempdir");
    let zip_path = dir.path().join("already.zip");

    // Build and write a buffer of length 200 including marker and signature
    let mut buf = Vec::with_capacity(200);
    buf.extend_from_slice(&vec![0u8; 100]);      // first 100 bytes
    buf.extend_from_slice(&MARKER);             // marker at pos 100
    buf.extend_from_slice(&vec![0u8; 64]);      // dummy “existing” signature
    fs::write(&zip_path, &buf).expect("write initial zip");

    // Attempt to sign: should error out.
    let err = sign_zip(zip_path.to_str().unwrap()).expect_err("should have failed");
    let msg = err.to_string();
    assert!(
        msg.contains("already contains the expected signature marker"),
        "Unexpected error message: {}",
        msg
    );

    // Ensure we did not overwrite the file.
    let after = fs::read(&zip_path).expect("read after");
    assert_eq!(after, buf, "File was modified despite error");
}

#[test]
fn test_verify_zip_success() {
    // Create a temp directory and a dummy ZIP file.
    let dir = tempdir().expect("failed to create tempdir");
    let zip_path = dir.path().join("foo.zip");
    fs::write(&zip_path, b"dummy-zip-content").expect("write initial zip");

    // Sign it using the existing sign_zip helper.
    sign_zip(zip_path.to_str().unwrap()).expect("sign_zip should succeed");

    // Verify ZIP signature
    let key_path = dir.path().join("foo.key");
    let result = verify_zip(zip_path.to_str().unwrap(), key_path.to_str().unwrap());
    assert!(result.is_ok(), "Expected verification to succeed, got {:?}", result);
}

#[test]
fn test_verify_zip_too_small() {
    // Create a file smaller than 64 bytes.
    let dir = tempdir().expect("failed to create tempdir");
    let zip_path = dir.path().join("small.zip");
    fs::write(&zip_path, vec![0u8; 10]).expect("write small file");

    // Create a dummy .key so open() won't fail later.
    let key_path = dir.path().join("small.key");
    fs::write(&key_path, vec![0u8; PUBLIC_KEY_LENGTH]).expect("write dummy key");

    // Attempt to verify: should error about too-small file.
    let err = verify_zip(zip_path.to_str().unwrap(), key_path.to_str().unwrap())
        .expect_err("Expected error for too-small file");
    let msg = err.to_string();
    assert!(
        msg.contains("File is too small to contain a signature"),
        "Unexpected error message: {}",
        msg
    );
}

#[test]
fn test_verify_zip_invalid_signature() {
    // Create a dummy “signed” ZIP: data + marker + signature by keypair A.
    let dir = tempdir().expect("failed to create tempdir");
    let zip_path = dir.path().join("bad.zip");
    let mut data = b"important-data".to_vec();
    data.extend_from_slice(&MARKER);

    // Sign with a new keypair A.
    let mut csprng = OsRng;
    let keypair_a: Keypair = Keypair::generate(&mut csprng);
    let sig = keypair_a.sign(&data).to_bytes();
    data.extend_from_slice(&sig);
    fs::write(&zip_path, &data).expect("write bad zip");

    // Write a .key file for another keypair B.
    let key_path = dir.path().join("bad.key");
    let keypair_b: Keypair = Keypair::generate(&mut csprng);
    fs::write(&key_path, keypair_b.public.to_bytes()).expect("write wrong key");

    // Attempt to verify with key from keypair B.
    let err = verify_zip(zip_path.to_str().unwrap(), key_path.to_str().unwrap())
        .expect_err("Expected signature verification failure");
    let msg = err.to_string();
    assert!(
        msg.contains("Signature verification failed"),
        "Unexpected error message: {}",
        msg
    );
}