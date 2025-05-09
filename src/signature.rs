use std::fs::File;
use std::fs;
use std::io::Read;
use ed25519_dalek::{Keypair, Signer, Signature, PublicKey, Verifier};
use rand::rngs::OsRng;
use std::io;

pub fn sign_zip(zip_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new keypair
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);

    // Read the ZIP file as byte array
    let mut file = File::open(zip_file_path)?;
    let mut zip_bytes = Vec::new();
    file.read_to_end(&mut zip_bytes)?;

    // Check for a specific marker or pattern
    let expected_marker: [u8; 4] = [0x05, 0x04, 0x07, 0x07];
    let marker_position = zip_bytes.len() as isize - 64 - expected_marker.len() as isize;
    if marker_position > 0 {
        let start = marker_position as usize;
        let end = start + expected_marker.len();
        let marker_bytes = &zip_bytes[start..end];
        if marker_bytes == expected_marker {
            return Err("File already contains the expected signature marker".into());
        }
    }

    // Append the marker bytes to the ZIP file bytes
    zip_bytes.extend_from_slice(&expected_marker);

    // Sign the ZIP file bytes
    let signature: Signature = keypair.sign(&zip_bytes);
    let signature_bytes = signature.to_bytes();

    // Append the signature bytes to the ZIP file bytes
    zip_bytes.extend_from_slice(&signature_bytes);

    fs::write(zip_file_path, &zip_bytes)?;
    fs::write(zip_file_path.replace(".zip", ".key"), keypair.public.to_bytes())?;

    Ok(()) 
}

pub fn verify_zip(zip_file_path: &str, verification_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the ZIP file into a byte array
    let mut file = File::open(zip_file_path)?;
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)?;

    // Check if the file is large enough
    if file_bytes.len() < 64 {
        return Err("File is too small to contain a signature".into());
    }

    // Check for a specific marker or pattern
    let expected_marker: [u8; 4] = [0x05, 0x04, 0x07, 0x07];
    let marker_position = file_bytes.len() as isize - 64 - expected_marker.len() as isize;
    if marker_position > 0 {
        let start = marker_position as usize;
        let end = start + expected_marker.len();
        let marker_bytes = &file_bytes[start..end];
        if marker_bytes != expected_marker {
            return Err("File already contains the expected signature marker".into());
        }
    }

    // Extract the last 64 bytes as the signature
    let signature_bytes = &file_bytes[file_bytes.len() - 64..];
    let signature = Signature::from_bytes(signature_bytes)?;

    // Read the remaining bytes as the data to verify
    let data_to_verify = &file_bytes[..file_bytes.len() - 64];

    // Read the public key from the .key file
    let mut public_key_file = File::open(verification_path)?;
    let mut public_key_bytes = Vec::new();
    public_key_file.read_to_end(&mut public_key_bytes)?;
    let public_key = PublicKey::from_bytes(&public_key_bytes)?;

    // Verify the signature
    public_key
        .verify(data_to_verify, &signature)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Signature verification failed"))?;

    Ok(())
}