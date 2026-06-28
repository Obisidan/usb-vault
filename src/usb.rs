//! USB device operations.
//!
//! Handles raw block-level read/write on Linux device nodes.
//! Uses direct I/O for performance and correctness.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::crypto::aes::aes256_ctr_process;
use crate::crypto::kdf::derive_key;
use crate::crypto::sha256::sha256_digest;

/// Vault header magic bytes
pub const VAULT_MAGIC: &[u8; 8] = b"USBVAULT";
/// Header version
pub const VAULT_VERSION: u8 = 1;
/// Salt size in bytes
pub const SALT_SIZE: usize = 16;
/// Verification tag size
pub const TAG_SIZE: usize = 32;
/// Nonce size for AES-256-CTR
pub const NONCE_SIZE: usize = 12;
/// Total header size: magic(8) + version(1) + salt(16) + nonce(12) + tag(32) = 69
pub const HEADER_SIZE: usize = 8 + 1 + SALT_SIZE + NONCE_SIZE + TAG_SIZE;

/// Vault header layout
#[derive(Debug, Clone)]
pub struct VaultHeader {
    pub magic: [u8; 8],
    pub version: u8,
    pub salt: [u8; 16],
    pub nonce: [u8; 12],
    pub tag: [u8; 32],
}

impl VaultHeader {
    pub fn new(salt: [u8; 16], nonce: [u8; 12], tag: [u8; 32]) -> Self {
        Self {
            magic: *VAULT_MAGIC,
            version: VAULT_VERSION,
            salt,
            nonce,
            tag,
        }
    }

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8] = self.version;
        buf[9..25].copy_from_slice(&self.salt);
        buf[25..37].copy_from_slice(&self.nonce);
        buf[37..69].copy_from_slice(&self.tag);
        buf
    }

    pub fn from_bytes(buf: &[u8; HEADER_SIZE]) -> Self {
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&buf[0..8]);
        let version = buf[8];
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&buf[9..25]);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&buf[25..37]);
        let mut tag = [0u8; 32];
        tag.copy_from_slice(&buf[37..69]);
        Self { magic, version, salt, nonce, tag }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == *VAULT_MAGIC && self.version == VAULT_VERSION
    }
}

/// Get the size of a device in bytes
pub fn device_size(path: &Path) -> Result<u64, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open device: {}", e))?;
    let metadata = file.metadata().map_err(|e| format!("Cannot read metadata: {}", e))?;
    Ok(metadata.len())
}

/// Read header from device
pub fn read_header(path: &Path) -> Result<VaultHeader, String> {
    let mut file = File::open(path).map_err(|e| format!("Cannot open device: {}", e))?;
    let mut buf = [0u8; HEADER_SIZE];
    file.read_exact(&mut buf).map_err(|e| format!("Cannot read header: {}", e))?;
    let header = VaultHeader::from_bytes(&buf);
    if !header.is_valid() {
        return Err("Not a valid USB Vault device (bad magic or version)".to_string());
    }
    Ok(header)
}

/// Write header to device
pub fn write_header(path: &Path, header: &VaultHeader) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("Cannot open device for writing: {}", e))?;
    file.write_all(&header.to_bytes())
        .map_err(|e| format!("Cannot write header: {}", e))?;
    file.flush().map_err(|e| format!("Flush failed: {}", e))?;
    Ok(())
}

/// Encrypt a device: derive key from password, write header, encrypt all data
pub fn encrypt_device(path: &Path, password: &str) -> Result<(), String> {
    // Generate random salt and nonce
    let salt = random_bytes_16();
    let nonce = random_bytes_12();

    // Derive key
    let key = derive_key(password, &salt);

    // Compute verification tag
    let mut tag_data = Vec::with_capacity(44);
    tag_data.extend_from_slice(&key);
    tag_data.extend_from_slice(&nonce);
    let tag = sha256_digest(&tag_data);

    let header = VaultHeader::new(salt, nonce, tag);

    // Get device size
    let dev_size = device_size(path)?;
    if dev_size < HEADER_SIZE as u64 {
        return Err("Device too small (less than header size)".to_string());
    }

    // Write header first
    write_header(path, &header)?;

    // Encrypt device data (everything after header)
    let data_size = (dev_size - HEADER_SIZE as u64) as usize;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("Cannot open device: {}", e))?;

    file.seek(SeekFrom::Start(HEADER_SIZE as u64))
        .map_err(|e| format!("Seek failed: {}", e))?;

    // Process in chunks to avoid memory issues with large drives
    const CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut written = 0usize;

    while written < data_size {
        let to_read = std::cmp::min(CHUNK_SIZE, data_size - written);
        let n = file.read(&mut buf[..to_read]).map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 { break; }

        let encrypted = aes256_ctr_process(&key, &nonce, &buf[..n]);

        file.seek(SeekFrom::Start((HEADER_SIZE + written) as u64))
            .map_err(|e| format!("Seek failed: {}", e))?;
        file.write_all(&encrypted)
            .map_err(|e| format!("Write failed: {}", e))?;

        written += n;
    }

    file.flush().map_err(|e| format!("Flush failed: {}", e))?;
    Ok(())
}

/// Decrypt a device: read header, derive key, verify, decrypt all data
pub fn decrypt_device(path: &Path, password: &str) -> Result<(), String> {
    let header = read_header(path)?;

    // Derive key from password + salt
    let key = derive_key(password, &header.salt);

    // Verify password
    let mut tag_data = Vec::with_capacity(44);
    tag_data.extend_from_slice(&key);
    tag_data.extend_from_slice(&header.nonce);
    let expected_tag = sha256_digest(&tag_data);

    if expected_tag != header.tag {
        return Err("Wrong password or corrupted header".to_string());
    }

    // Get device size
    let dev_size = device_size(path)?;
    let data_size = (dev_size - HEADER_SIZE as u64) as usize;

    // Decrypt in place
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("Cannot open device: {}", e))?;

    file.seek(SeekFrom::Start(HEADER_SIZE as u64))
        .map_err(|e| format!("Seek failed: {}", e))?;

    const CHUNK_SIZE: usize = 1024 * 1024;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut written = 0usize;

    while written < data_size {
        let to_read = std::cmp::min(CHUNK_SIZE, data_size - written);
        let n = file.read(&mut buf[..to_read]).map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 { break; }

        let decrypted = aes256_ctr_process(&key, &header.nonce, &buf[..n]);

        file.seek(SeekFrom::Start((HEADER_SIZE + written) as u64))
            .map_err(|e| format!("Seek failed: {}", e))?;
        file.write_all(&decrypted)
            .map_err(|e| format!("Write failed: {}", e))?;

        written += n;
    }

    file.flush().map_err(|e| format!("Flush failed: {}", e))?;
    Ok(())
}

/// Wipe a device: overwrite header with zeros (destroys vault, makes it a blank device)
pub fn wipe_device(path: &Path) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("Cannot open device: {}", e))?;

    // Write zeros to header area
    let zeros = vec![0u8; HEADER_SIZE];
    file.write_all(&zeros).map_err(|e| format!("Wipe failed: {}", e))?;
    file.flush().map_err(|e| format!("Flush failed: {}", e))?;
    Ok(())
}

/// Check if a device is already encrypted with usb-vault
pub fn is_encrypted(path: &Path) -> bool {
    read_header(path).is_ok()
}

/// Generate 16 random bytes using getrandom
fn random_bytes_16() -> [u8; 16] {
    let mut buf = [0u8; 16];
    getrandom::getrandom(&mut buf).expect("CSPRNG failure");
    buf
}

/// Generate 12 random bytes using getrandom
fn random_bytes_12() -> [u8; 12] {
    let mut buf = [0u8; 12];
    getrandom::getrandom(&mut buf).expect("CSPRNG failure");
    buf
}
