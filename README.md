<div align="center">

<img src="https://github.com/Obisidan/Obisidan/raw/main/sayori_halfbody.webp" width="200"/>

### pure rust encryption suite — lock any USB behind a password

[![License: MIT](https://img.shields.io/badge/License-MIT-FF69B4?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-8%2F8-brightgreen?style=flat-square)](https://github.com/Obisidan/usb-vault)
[![Rust](https://img.shields.io/badge/Rust-0D1117?style=flat-square&logo=rust&logoColor=orange)](https://www.rust-lang.org/)

</div>

## Features

- **AES-256-CTR** — pure Rust, zero external crypto crates
- **PBKDF2-HMAC-SHA256** — 600k iterations (OWASP recommended)
- **TUI interface** — neon-dark themed, keyboard navigable
- **Permanent encryption** — wrong password = no data. No backdoor.
- **Paper backup reminder** — warns you to write down your password before exit
- **CLI mode** — scriptable encrypt/decrypt/wipe from command line

## Security Model

1. You provide a **password** and select a **USB device**
2. A random 16-byte **salt** and 12-byte **nonce** are generated
3. Password stretched via **PBKDF2-HMAC-SHA256** (600k iterations) into 256-bit AES key
4. **Verification tag** (SHA-256 of key+nonce) confirms correct password
5. Header written to sector 0, **all remaining data** encrypted in-place with AES-256-CTR

Without the correct password, the verification tag won't match. Data is irrecoverable. Only option: format the drive.

## Installation

```bash
git clone https://github.com/Obisidan/usb-vault.git
cd usb-vault
cargo build --release
```

## Usage

### Interactive TUI

```bash
sudo ./target/release/usb-vault
```

Navigate with `j/k` or arrow keys. Press `Enter` to select. Press `q` or `Esc` to quit (warning will appear).

### CLI Mode

```bash
# Encrypt (interactive password prompt)
sudo ./target/release/usb-vault encrypt --device /dev/sdX

# Decrypt
sudo ./target/release/usb-vault decrypt --device /dev/sdX

# Wipe vault header (destroys encrypted data permanently)
sudo ./target/release/usb-vault wipe --device /dev/sdX
```

Find your USB device:
```bash
lsblk
sudo umount /dev/sdb*
```

## Architecture

```
usb-vault/
├── src/
│   ├── main.rs          # Entry point, CLI argument parsing
│   ├── lib.rs           # Module declarations
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── aes.rs       # AES-256-CTR (FIPS-197, SP800-38A)
│   │   ├── sha256.rs    # SHA-256 (FIPS-180-4)
│   │   └── kdf.rs       # PBKDF2-HMAC-SHA256 (600k iterations)
│   ├── usb.rs           # Device I/O, header management
│   ├── tui.rs           # Ratatui frontend
│   └── warning.rs       # Exit warnings, paper backup reminders
├── Cargo.toml
├── LICENSE
└── README.md
```

## ⚠️ Warning

> **IF YOU LOSE YOUR PASSWORD, YOUR DATA IS GONE FOREVER.**
>
> The authors are not responsible for any data loss. Always test with a non-critical device first.
> Write your password down on paper and store it somewhere safe.

---

Sayori is the best character ever. If you know, you know.
