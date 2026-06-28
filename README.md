<div align="center">

<img src="https://camo.githubusercontent.com/ab202e134a0dd375ea52856fce7ddfd62416d50de493529a695378b358813d24/68747470733a2f2f63617073756c652d72656e6465722d76657263656c2e6170702f6170693f747970653d7265637426636f6c6f723d303a3044313131372c35303a4646363942342c3130303a304431313137266865696768743d32" width="100%" />

<h1>USB Vault</h1>

<b>pure rust encryption suite — lock any USB behind a password</b>

<br/>

<img src="https://camo.githubusercontent.com/6f5b6cb8c343718d574992fd7b0f226075fe0d00634940be484bb4baf5dba962/68747470733a2f2f726561646d652d747970696e672d7376672e64656d6f6c61622e636f6f6e743d464636394234267765696768743d3530302673697a653d3234266475726174696f6e3d313230302670617573653d36303026636f6f6c6f723d46463639423426656e7465723d74727565267663696570617573653d3630302677696474683d343030266c696e65733d253246212535432b6f6269736964616e2e6465763b7361796f72692b69732b7468652b626573742b6368617261637465722b657665723b727573742b2532462532462b7a65726f2b646570733b33616d2b766962652b636f64696e67" width="400" align="center"/>

<br/>

<a href="https://github.com/Obisidan/usb-vault"><img src="https://camo.githubusercontent.com/e0bd78c9883f788a722d5746b3ecfad92046a788d1b5abfda0bdac48983edb9f/68747470733a2f2f6769746875622d726561646d652d73746174732e76657263656c2e6170702f6170692f70696e3f757365726e616d653d4f6269736964616e267265703d636970686572267468656d653d7261646963616c2662675f636f6c6f723d304431313137267469746c655f636f6c6f723d364636394234266578745f636f6c6f723d36666330636226626f626f726465725f636f6c6f723d364636394234323226686964655f626f6f726465723d74727565266e636f636f6c6f723d364636394234266578745f636f6c6f723d3666633063622662686964655f626f6f726465723d7472756526636f6f6b6579733d34266d617267696e2d773d3130266d617267696e2d683d3130" width="200"/></a>
<a href="https://discord.gg/alexdagreatest2"><img src="https://camo.githubusercontent.com/53dcd253445a758c89b3d2c94a981ee407d3b17acc2fc297cf12f720adbfd782/68747470733a2f2f696d67736869656c64732e696f2f62616467652f646973636f72642d3044313131373f7374796c653d666c61742d737175617265266c6f676f3d646973636f72642666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666636f6c6f723d364636394234" width="67"/></a>

</div>

---

<div align="center">

<a href="https://github.com/Obisidan"><img src="https://camo.githubusercontent.com/bc99dd1537dba5d6c87099ea590d9f16022230bc3eff07e861a77a7a4512a3cf/68747470733a2f2f6769746875622d726561646d652d73746174732e76657263656c2e6170702f6170692f757365726e616d653d4f6269736964616e2673686f775f69636f6e733d74727565267468656d653d7261646963616c2662675f636f6c6f723d304431313137267469746c655f636f6c6f723d364636394234266578745f636f6c6f723d36666330636226626f626f726465725f636f6c6f723d364636394234323226686964655f626f6f726465723d74727565266e636f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342664636f6e745f636f6c6f723d36666330636226686964655f626f6f726465723d74727565266e636f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342664636f6e745f636f6c6f723d36666330636226626f626f726465723d74727565266e636f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342664636f6e745f636f6c6f723d3666633063622646366463655f626f6f726465723d74727565266e636f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342664636f6e745f636f6c6f723d366633063622662686964655f626f6f726465723d74727565266e636f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342664636f6e745f636f6c6f723d36666330636226626f6f726465723d74727565266e636f636f6c6f723d3646363942342669636f6e5f636f6c6f723d3646363942342664636f6e745f636f6c6f723d3666633063622664636463655f626f6f726465723d747275652665636f6c6f723d3426d617267696e2d773d3130266d617267696e2d683d3130" width="200"/></a>

</div>

<div align="center">

![](https://img.shields.io/badge/Rust-0D1117?style=for-the-badge&logo=rust&logoColor=FF69B4)
[![License: MIT](https://img.shields.io/badge/License-MIT-FF69B4.svg?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-8%2F8%20Passing-00FF00?style=for-the-badge)](https://github.com/Obisidan/usb-vault)

</div>

---

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

<div align="center">

Built by [Obisidan](https://github.com/Obisidan) — pure Rust, zero dependencies, late nights in Ohio.

Sayori is the best character ever. If you know, you know.

</div>
