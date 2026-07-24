<div align="center">

# MovieBox-Tui

A lightning-fast terminal client for discovering and streaming movies, TV shows, and anime directly from your keyboard.

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

**MovieBox-Tui** brings the cinematic streaming experience to your terminal. It directly queries the MovieBox API to search catalogs, resolve video streams, and seamlessly hand them off to your favorite local video player (`mpv`, `IINA`, or `VLC`). 

No browsers, no ads, no configuration. Type a title, select your preferred quality, and watch.

> **Disclaimer:** This project is a third-party client. It does not host, store, or redistribute any media. It strictly resolves links provided by the upstream API. It is intended for educational and personal use only. You are responsible for complying with copyright law in your jurisdiction.

---

## Features

- **Live Search & Discovery:** Type to search instantly, or use slash commands (`/movies`, `/shows`, `/anime`, `/discover`) to browse trending feeds.
- **Rich Metadata:** Hover over any title to instantly view full-resolution posters, release years, IMDb ratings, genres, and a complete synopsis.
- **Instant Playback:** Fetches multiple stream qualities (1080p, 720p, 480p) in parallel. Choose your resolution, pick subtitle tracks, and start watching in a single keystroke.
- **Lightning Fast Native Downloader:** Save videos for offline viewing using up to 16 parallel connections to maximize your bandwidth. Features live speed, ETA, and progress tracking.
- **Universal Terminal Support:** Works beautifully on any terminal and across all major operating systems (macOS, Linux, and Windows). Posters will render as native images in supported terminals or gracefully fall back to text.

---

## Screenshots

<details open>
<summary><b>Home and search</b></summary>
<br>

| Home | Search results |
| :---: | :---: |
| <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/01-home.jpg" alt="Home screen" width="480"> | <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/02-search-results.jpg" alt="Search results" width="480"> |

</details>

<details>
<summary><b>Details view</b></summary>
<br>

| Movie | TV series |
| :---: | :---: |
| <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/03-movie-details.jpg" alt="Movie details" width="480"> | <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/04-series-details.jpg" alt="Series details" width="480"> |

</details>

<details>
<summary><b>Discover feeds</b></summary>
<br>

| Movies | Series | Anime |
| :---: | :---: | :---: |
| <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/05-discover-movies.jpg" alt="Discover movies" width="320"> | <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/06-discover-series.jpg" alt="Discover series" width="320"> | <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/07-discover-anime.jpg" alt="Discover anime" width="320"> |

</details>

---

## Installation

You can install MovieBox-Tui using our automated scripts, or via Cargo if you are a Rust developer.

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.ps1 | iex
```

### Via Cargo
```bash
cargo install moviebox-tui
```

---

## Requirements

1. **Terminal:** Any terminal! A minimum window size of 85×24 is required. Modern terminals with graphics protocols will display native posters automatically.
2. **Video Player:** `mpv`, `IINA` (macOS), or `VLC` must be installed on your system. MovieBox-Tui will auto-detect them.

<details>
<summary><b>Download pre-built binary</b></summary>
<br>

You can download the compiled executable directly from the [Releases page](https://github.com/mesamirh/MovieBox-Tui/releases/latest).

</details>

<details>
<summary><b>Need to install a video player?</b></summary>
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

## Getting Started

Launch the app by typing `moviebox-tui` in your terminal.

- **Search:** Start typing to search for titles instantly.
- **Navigate:** Use the <kbd>Up</kbd> / <kbd>Down</kbd> / <kbd>Left</kbd> / <kbd>Right</kbd> arrow keys to move around.
- **Select:** Press <kbd>Enter</kbd> to view details, browse seasons/episodes, and start playback.
- **Switch Player:** Press <kbd>o</kbd> on a stream to choose a different installed player.
- **Download:** Press <kbd>d</kbd> to download the selected stream to your machine.
- **Refresh:** Press <kbd>r</kbd> to re-resolve expired stream links.
- **Help:** Press <kbd>?</kbd> anywhere in the app to view all keybindings.
- **Quit:** Press <kbd>q</kbd> or <kbd>Esc</kbd> repeatedly to exit.

### Slash Commands
Use the search bar to run dedicated slash commands:
- `/discover`, `/home` - Browse trending and curated titles.
- `/movies`, `/shows`, `/anime` - Jump to specific category feeds.
- `/clear-cache` - Free up disk space by removing cached data and images.
- `/update` - Manually check for updates.
- `/toggle-update` - Enable or disable background update checking.

---

## Contributing

Contributions are highly welcome! For major changes, please open an issue first to discuss your ideas.

```bash
git clone https://github.com/<your-username>/MovieBox-Tui.git
cd MovieBox-Tui
cargo build
```

We follow [Conventional Commits](https://www.conventionalcommits.org/). Ensure `cargo fmt` and `cargo clippy` pass cleanly before submitting a Pull Request. See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

---

## Support

If you enjoy this project and want to support its continuous development:

- **EVM (ETH, BNB, Polygon, etc):** `0x7ea20d5fa29d87f33195f5a3b211ff94038d794c`
- **BTC:** `3MEAtqtRWrQBhnaMi3Zuf5nt2efNUS2LUQ`
- **LTC:** `ltc1qhjkq2n6tsayxj56n3c53uqv23v8vqhvc9g3vxl`

---

<div align="center">

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.<br>
Made by [**@mesamirh**](https://github.com/mesamirh)

<sub>Not affiliated with MovieBox or its operators.</sub>

</div>
