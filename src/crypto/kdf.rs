//! PBKDF2-HMAC-SHA256 key derivation.
//! 
//! Derives a 256-bit AES key from a user password + salt using PBKDF2.
//! 600,000 iterations (OWASP recommendation for PBKDF2-SHA256 as of 2023).

use super::sha256::sha256_digest;

const PBKDF2_ITERATIONS: u32 = 600_000;

/// HMAC-SHA256 implementation
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    const IPAD: u8 = 0x36;
    const OPAD: u8 = 0x5c;

    let mut padded_key = [0u8; 64];
    if key.len() <= 64 {
        padded_key[..key.len()].copy_from_slice(key);
    } else {
        let hash = sha256_digest(key);
        padded_key[..32].copy_from_slice(&hash);
    }

    let inner_len = 64 + data.len();
    let mut inner_input = vec![0u8; inner_len];
    for (i, b) in padded_key.iter().enumerate() {
        inner_input[i] = b ^ IPAD;
    }
    inner_input[64..].copy_from_slice(data);
    let inner_hash = sha256_digest(&inner_input);

    let mut outer_input = [0u8; 96];
    for (i, b) in padded_key.iter().enumerate() {
        outer_input[i] = b ^ OPAD;
    }
    outer_input[64..96].copy_from_slice(&inner_hash);

    sha256_digest(&outer_input)
}

/// PBKDF2-HMAC-SHA256: derive a `dk_len` byte key from password + salt.
pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], dk_len: usize) -> Vec<u8> {
    let h_len = 32usize;
    let l = (dk_len + h_len - 1) / h_len;
    let mut derived_key = vec![0u8; l * h_len];

    for block in 1..=l {
        let block_bytes = (block as u32).to_be_bytes();
        let mut combined = Vec::with_capacity(salt.len() + 4);
        combined.extend_from_slice(salt);
        combined.extend_from_slice(&block_bytes);
        let mut u = hmac_sha256(password, &combined);
        let mut t = u;

        for _ in 1..PBKDF2_ITERATIONS {
            u = hmac_sha256(password, &u);
            for (t_byte, u_byte) in t.iter_mut().zip(u.iter()) {
                *t_byte ^= *u_byte;
            }
        }

        let offset = (block - 1) * h_len;
        derived_key[offset..offset + h_len].copy_from_slice(&t);
    }

    derived_key[..dk_len].to_vec()
}

/// Derive a 32-byte AES-256 key from password + salt.
pub fn derive_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let dk = pbkdf2_sha256(password.as_bytes(), salt, 32);
    let mut key = [0u8; 32];
    key.copy_from_slice(&dk);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_deterministic() {
        let key = b"secret";
        let msg = b"hello world";
        let tag = hmac_sha256(key, msg);
        assert_ne!(tag, [0u8; 32]);
        let tag2 = hmac_sha256(key, msg);
        assert_eq!(tag, tag2);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let salt = [0xABu8; 16];
        let k1 = derive_key("correct horse battery staple", &salt);
        let k2 = derive_key("correct horse battery staple", &salt);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_derive_key_differs_with_password() {
        let salt = [0x01u8; 16];
        let k1 = derive_key("password1", &salt);
        let k2 = derive_key("password2", &salt);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_derive_key_differs_with_salt() {
        let salt1 = [0x01u8; 16];
        let salt2 = [0x02u8; 16];
        let k1 = derive_key("same password", &salt1);
        let k2 = derive_key("same password", &salt2);
        assert_ne!(k1, k2);
    }
}
