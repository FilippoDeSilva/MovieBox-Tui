<div align="center">

# MovieBox-Tui

Stream and download movies, TV shows, and anime from the terminal.

[![Crates.io](https://img.shields.io/crates/v/moviebox-tui.svg?logo=rust)](https://crates.io/crates/moviebox-tui)
[![Downloads](https://img.shields.io/crates/d/moviebox-tui.svg)](https://crates.io/crates/moviebox-tui)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg?logo=rust)](#requirements)

<br>

<p align="center">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/01-home.jpg" alt="MovieBox-Tui home screen" width="88%"/>
</p>

**[Watch the demo on YouTube](https://youtu.be/0L1Wc3cwMCc)**

</div>

---

## What is this?

**MovieBox-Tui** is a terminal-based streaming client. It interfaces with the MovieBox API to search catalogs, resolve stream URLs, and open them in your local video player (`mpv`, `IINA`, or `VLC`). 

No browser or configuration is required. Search for a title, select a video quality, and press enter to play or download the stream.

> **Disclaimer:** This project is a third-party client. It does not host, store, or redistribute any media. It strictly resolves links provided by the upstream API. It is intended for educational and personal use only. You are responsible for complying with copyright law in your jurisdiction.

---

## Screenshots

<details open>
<summary><b>Home and search</b></summary>
<br>

<p align="center">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/01-home.jpg" alt="Home screen" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/02-search-results.jpg" alt="Search results" width="49%">
</p>

</details>

<details>
<summary><b>Details view</b></summary>
<br>

<p align="center">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/03-movie-details.jpg" alt="Movie details" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/04-series-details.jpg" alt="Series details" width="49%">
</p>

</details>

<details>
<summary><b>Discover feeds</b></summary>
<br>

<p align="center">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/05-discover-movies.jpg" alt="Discover movies" width="32%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/06-discover-series.jpg" alt="Discover series" width="32%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/07-discover-anime.jpg" alt="Discover anime" width="32%">
</p>

</details>

---

## Features

- **Search & discover easily:** Just start typing to find what you want, or use commands like `/movies` and `/anime` to see what's trending.
- **All the details you need:** Select a title to see its poster, IMDb rating, release year, genre, and a quick summary.
- **Play instantly:** The app grabs different stream qualities (like 1080p and 720p) behind the scenes. Pick your resolution and subtitles, and it starts playing immediately.
- **Built-in fast downloader:** 
  - Download videos straight to your disk using multiple connections to max out your internet speed. You get live speed and ETA updates while you wait.
  - **Subtitles included:** It automatically grabs the `.srt` subtitle files to match your video.
  - **Download entire seasons:** Queue up a whole season at once. Just tell it what language you want for the subtitles, and it handles the rest automatically.
- **Works on your terminal:** Whether you're on macOS, Linux, or Windows, it just works. If your terminal supports it, you'll get actual image posters; otherwise, it falls back nicely to text.

---

## What you'll need

1. **A Terminal:** Any terminal works! Just make sure your window is at least 85×24 characters. If you use a modern terminal, you'll even get high quality native image posters automatically.
2. **A Video Player:** You'll need `mpv`, `IINA` (for macOS), or `VLC` installed. The app will automatically find whichever one you have and use it.

<details>
<summary><b>Want to download a pre-built binary instead?</b></summary>
<br>

You can grab the compiled executable directly from the [Releases page](https://github.com/mesamirh/MovieBox-Tui/releases/latest).

</details>

<details>
<summary><b>Need help installing a video player?</b></summary>
<br>

```bash
# macOS
brew install mpv
# or
brew install --cask iina

# Debian / Ubuntu
sudo apt install mpv

# Arch Linux
sudo pacman -S mpv

# Windows
choco install mpv
```

</details>

---

## Installation

The easiest way to get started is by using the quick install scripts below. If you're a Rust developer, you can also just grab it from Cargo.

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.sh | bash
```

### Windows
```powershell
powershell -c "irm https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.ps1 | iex"
```

### Cargo (Rust)
```bash
cargo install moviebox-tui
```

---

## Getting Started

Just open your terminal and type `moviebox-tui` to jump in.

Here's how to get around:
- **Search:** Just start typing! It searches instantly.
- **Navigate:** Use the <kbd>Up</kbd> / <kbd>Down</kbd> / <kbd>Left</kbd> / <kbd>Right</kbd> arrow keys.
- **Select:** Press <kbd>Enter</kbd> to view details, pick episodes, or start playing a video.
- **Switch Player:** Press <kbd>o</kbd> on a stream if you want to open it in a different video player.
- **Download:** Press <kbd>d</kbd> on any episode or season to save it to your disk (you'll get to pick your subtitles first).
- **Refresh:** Press <kbd>r</kbd> if a stream link expires and needs to be re-fetched.
- **Help:** Need a quick reminder? Press <kbd>?</kbd> anywhere to see the cheat sheet.
- **Quit:** Press <kbd>q</kbd> or mash <kbd>Esc</kbd> to exit.

### Handy Slash Commands
You can type these special commands straight into the search bar:
- `/discover` or `/home` - See what's trending right now.
- `/movies`, `/shows`, `/anime` - Jump straight to a specific category.
- `/clear-cache` - Free up disk space by wiping downloaded images and cache.
- `/update` - Check to see if there's a new version of the app.
- `/toggle-update` - Turn automatic background update checking on or off.

---

## Contributing

I'd love your help making this even better! If you've got a big feature in mind, it's usually best to open an issue first so we can chat about it.

```bash
git clone https://github.com/<your-username>/MovieBox-Tui.git
cd MovieBox-Tui
cargo build
```

Just try to follow [Conventional Commits](https://www.conventionalcommits.org/) and make sure `cargo fmt` and `cargo clippy` are happy before you open a PR. You can check out [CONTRIBUTING.md](CONTRIBUTING.md) for the full rundown.

---

## Support

If this TUI makes your movie nights better and you want to support its development, crypto tips are always super appreciated:

- **EVM (ETH, BNB, Polygon, etc):** `0x7ea20d5fa29d87f33195f5a3b211ff94038d794c`
- **BTC:** `3MEAtqtRWrQBhnaMi3Zuf5nt2efNUS2LUQ`
- **LTC:** `ltc1qhjkq2n6tsayxj56n3c53uqv23v8vqhvc9g3vxl`

---

<div align="center">

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.<br>
Built by [**@mesamirh**](https://github.com/mesamirh)

<sub>Not affiliated with MovieBox or its operators.</sub>

</div>
