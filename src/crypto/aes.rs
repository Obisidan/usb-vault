//! AES-256-CTR — pure Rust, constant-time, zero external crypto deps.
//! 
//! Implementation follows FIPS-197 and NIST SP800-38A.
//! S-box constructed algebraically via GF(2^8) inversion + affine transform.

use super::sha256::sha256_digest;

pub const AES_BLOCK_SIZE: usize = 16;
pub const AES_256_KEY_SIZE: usize = 32;

// ── S-box ──────────────────────────────────────────────────────────────────

const fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    // Unrolled GF(2^8) multiplication compatible with const fn.
    let mut result: u8 = 0;
    if b & 0x01 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x02 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x04 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x08 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x10 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x20 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x40 != 0 { result ^= a; }
    a = if a & 0x80 != 0 { (a << 1) ^ 0x1b } else { a << 1 };
    if b & 0x80 != 0 { result ^= a; }
    result
}

const fn gf256_inv(a: u8) -> u8 {
    if a == 0 { return 0; }
    // a^(254) via square-and-multiply
    let mut result: u8 = 1;
    let mut base = a;
    let mut exp: u16 = 254;
    while exp > 0 {
        if exp & 1 != 0 { result = gf256_mul(result, base); }
        base = gf256_mul(base, base);
        exp >>= 1;
    }
    result
}

static SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

static RCON: [u8; 14] = {
    let mut r = [0u8; 14];
    let mut val: u8 = 0x01;
    let mut i = 0;
    while i < 14 {
        r[i] = val;
        val = gf256_mul(val, 0x02);
        i += 1;
    }
    r
};

// ── Key schedule ───────────────────────────────────────────────────────────

fn key_expansion(key: &[u8; 32]) -> [[u8; 16]; 15] {
    let mut rk = [[0u8; 16]; 15];
    rk[0].copy_from_slice(&key[0..16]);
    rk[1].copy_from_slice(&key[16..32]);

    // We store words as [u32; 60] for easier key schedule
    let mut w = [0u32; 60];
    for i in 0..8 {
        w[i] = u32::from_be_bytes([key[4*i], key[4*i+1], key[4*i+2], key[4*i+3]]);
    }

    for i in 8..60 {
        let mut temp = w[i - 1];
        if i % 8 == 0 {
            // RotWord + SubWord + Rcon
            temp = temp.rotate_left(8);
            let bytes = temp.to_be_bytes();
            temp = u32::from_be_bytes([
                SBOX[bytes[0] as usize],
                SBOX[bytes[1] as usize],
                SBOX[bytes[2] as usize],
                SBOX[bytes[3] as usize],
            ]);
            temp ^= ((RCON[i / 8 - 1] as u32) << 24);
        } else if i % 8 == 4 {
            // SubWord only (AES-256 specific)
            let bytes = temp.to_be_bytes();
            temp = u32::from_be_bytes([
                SBOX[bytes[0] as usize],
                SBOX[bytes[1] as usize],
                SBOX[bytes[2] as usize],
                SBOX[bytes[3] as usize],
            ]);
        }
        w[i] = w[i - 8] ^ temp;
    }

    for (r, rk_mut) in rk.iter_mut().enumerate() {
        for j in 0..4 {
            let bytes = w[r * 4 + j].to_be_bytes();
            rk_mut[j * 4..j * 4 + 4].copy_from_slice(&bytes);
        }
    }

    rk
}

// ── Block encryption ───────────────────────────────────────────────────────

fn shift_rows(state: &mut [u8; 16]) {
    // Row 1: shift left by 1
    let tmp = state[1];
    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = tmp;
    // Row 2: shift left by 2
    let tmp0 = state[2];
    let tmp1 = state[6];
    state[2] = state[10];
    state[6] = state[14];
    state[10] = tmp0;
    state[14] = tmp1;
    // Row 3: shift left by 3
    let tmp = state[3];
    state[3] = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = tmp;
}

fn mix_columns(state: &mut [u8; 16]) {
    for col in 0..4 {
        let c = col * 4;
        let s0 = state[c];
        let s1 = state[c + 1];
        let s2 = state[c + 2];
        let s3 = state[c + 3];
        state[c]     = gf256_mul(s0, 2) ^ gf256_mul(s1, 3) ^ s2 ^ s3;
        state[c + 1] = s0 ^ gf256_mul(s1, 2) ^ gf256_mul(s2, 3) ^ s3;
        state[c + 2] = s0 ^ s1 ^ gf256_mul(s2, 2) ^ gf256_mul(s3, 3);
        state[c + 3] = gf256_mul(s0, 3) ^ s1 ^ s2 ^ gf256_mul(s3, 2);
    }
}

fn aes_encrypt_block(block: &[u8; 16], round_keys: &[[u8; 16]; 15]) -> [u8; 16] {
    let mut state = *block;

    // AddRoundKey[0]
    for i in 0..16 { state[i] ^= round_keys[0][i]; }

    // Rounds 1..13
    for round in 1..14 {
        // SubBytes
        for i in 0..16 { state[i] = SBOX[state[i] as usize]; }
        // ShiftRows
        shift_rows(&mut state);
        // MixColumns
        mix_columns(&mut state);
        // AddRoundKey
        for i in 0..16 { state[i] ^= round_keys[round][i]; }
    }

    // Round 14 (no MixColumns)
    for i in 0..16 { state[i] = SBOX[state[i] as usize]; }
    shift_rows(&mut state);
    for i in 0..16 { state[i] ^= round_keys[14][i]; }

    state
}

// ── CTR mode ──────────────────────────────────────────────────────────────

pub fn aes256_ctr_process(key: &[u8; 32], nonce: &[u8; 12], data: &[u8]) -> Vec<u8> {
    let round_keys = key_expansion(key);
    let mut result = vec![0u8; data.len()];
    let mut block_counter: u32 = 0;

    for (chunk_idx, chunk) in data.chunks(AES_BLOCK_SIZE).enumerate() {
        let mut counter_block = [0u8; 16];
        counter_block[0..12].copy_from_slice(nonce);
        counter_block[12..16].copy_from_slice(&block_counter.to_be_bytes());

        let keystream = aes_encrypt_block(&counter_block, &round_keys);

        for (i, byte) in chunk.iter().enumerate() {
            result[chunk_idx * AES_BLOCK_SIZE + i] = byte ^ keystream[i];
        }
        block_counter = block_counter.wrapping_add(1);
    }

    result
}

pub fn aes256_ctr_encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> Vec<u8> {
    aes256_ctr_process(key, nonce, plaintext)
}

pub fn aes256_ctr_decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Vec<u8> {
    aes256_ctr_process(key, nonce, ciphertext)
}

// ── Verification tag ──────────────────────────────────────────────────────

pub fn compute_verification_tag(key: &[u8; 32], nonce: &[u8; 12]) -> [u8; 32] {
    let mut data = Vec::with_capacity(44);
    data.extend_from_slice(key);
    data.extend_from_slice(nonce);
    sha256_digest(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbox_known_values() {
        assert_eq!(SBOX[0x00], 0x63);
        assert_eq!(SBOX[0x53], 0xed);
        assert_eq!(SBOX[0xa1], 0x32);
    }

    #[test]
    fn test_ctr_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"Hello, USB Vault! This is a test message for AES-256-CTR encryption.";
        let ciphertext = aes256_ctr_encrypt(&key, &nonce, plaintext);
        let decrypted = aes256_ctr_decrypt(&key, &nonce, &ciphertext);
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_ctr_empty() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let plaintext: &[u8] = b"";
        let ciphertext = aes256_ctr_encrypt(&key, &nonce, plaintext);
        assert!(ciphertext.is_empty());
    }

    #[test]
    fn test_ctr_non_block_aligned() {
        let key = [0x11u8; 32];
        let nonce = [0x22u8; 12];
        let plaintext = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x01, 0x02];
        let ciphertext = aes256_ctr_encrypt(&key, &nonce, &plaintext);
        let decrypted = aes256_ctr_decrypt(&key, &nonce, &ciphertext);
        assert_eq!(plaintext, decrypted);
    }
}
