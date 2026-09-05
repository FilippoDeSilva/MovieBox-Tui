# Installation

MovieBox-TUI is available across macOS, Linux, Windows, and Android (Termux).

---

## macOS & Linux

Install via the automated script:

```bash
curl -fsSL https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.sh | bash
```

### macOS via Homebrew

```bash
brew tap mesamirh/moviebox-tui https://github.com/mesamirh/MovieBox-Tui
brew trust mesamirh/moviebox-tui
brew install moviebox-tui
```

---

## Windows

Install via PowerShell (run in Windows Terminal or PowerShell 5.1+):

```powershell
irm https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.ps1 | iex
```

---

## Android (Termux)

Install Termux tools, run the installer script, and grant storage permissions:

```bash
pkg update && pkg install termux-tools
curl -fsSL https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.sh | bash
termux-setup-storage
```

---

## Cargo (Crates.io)

Install directly using Cargo:

```bash
cargo install moviebox-tui --locked
```

---

## Compile from Source

Clone the repository and build the release binary:

```bash
git clone https://github.com/mesamirh/MovieBox-Tui.git
cd MovieBox-Tui
cargo build --release --locked
```

The compiled binary will be located at `target/release/moviebox-tui`.

---

## Verify Release Integrity

All release assets include cryptographically signed SHA-256 checksums and GitHub provenance attestations:

```bash
sha256sum -c SHA256SUMS --ignore-missing
gh attestation verify <archive-file> -R mesamirh/MovieBox-Tui
```
