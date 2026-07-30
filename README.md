<div align="center">

# MovieBox-TUI

**Stream movies, shows, anime, and live TV from your terminal.** <br>
Fast and clean. No configuration, no torrents, and no debrid required.

[![Crates.io](https://img.shields.io/crates/v/moviebox-tui.svg?logo=rust)](https://crates.io/crates/moviebox-tui)
[![Downloads](https://img.shields.io/crates/d/moviebox-tui.svg)](https://crates.io/crates/moviebox-tui)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg?logo=rust)](#requirements)

<br>

<img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/01-home-blocky.jpg" alt="MovieBox-TUI Home" width="85%">

**[See what's new in v0.1.7 on YouTube](https://youtu.be/5M2_mjH5r5Y)**

<sub>Found a bug? [Open an issue](https://github.com/mesamirh/MovieBox-Tui/issues) so I can fix it for everyone!</sub>

</div>


## Screenshots

<details>
<summary><b>Click to view gallery</b></summary>
<br>

<p align="center">
  <b>Movie & Series Details</b><br>
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/07-movie-details.jpg" alt="Movie Details" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/08-series-details.jpg" alt="Series Details" width="49%">
</p>

<p align="center">
  <b>Search & Downloads</b><br>
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/06-search-results.jpg" alt="Search Results" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/12-download-progress.jpg" alt="Download Progress" width="49%">
</p>

<p align="center">
  <b>Playback & Subtitles</b><br>
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/11-player-picker.jpg" alt="Media Player Selection" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/10-playback-subtitles.jpg" alt="Subtitle Language Selection" width="49%">
</p>

<p align="center">
  <b>Live TV Experience</b><br>
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/09-live-tv-list.jpg" alt="Live TV Channels" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/05-tv-help.jpg" alt="Live TV Configuration" width="49%">
</p>

<p align="center">
  <b>Home Themes</b><br>
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/03-home-3d.jpg" alt="3D Block Theme" width="49%">
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/02-home-ascii.jpg" alt="Minimal ASCII Theme" width="49%">
</p>

<p align="center">
  <b>Help & Configuration</b><br>
  <img src="https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/assets/screenshots/04-global-help.jpg" alt="Global Help Menu" width="49%">
</p>

</details>


## Features

### Streaming & Playback
- **Instant Search & Catalogs:** Type to search instantly, or browse trending movies, shows, and anime using slash commands (e.g., `/movies`, `/anime`).
- **Seamless Local Playback:** Resolves 4K/1080p streams and opens them instantly in your preferred local video player (`mpv`, `IINA`, or `VLC`).
- **Integrated Subtitles:** Automatically fetches available subtitles and lets you select your preferred language before playback.
- **Live IPTV:** Press `Ctrl+T` to toggle Live TV mode and stream thousands of live television channels globally.

### Advanced Downloading
- **Batch Season Downloader:** Queue up entire television seasons for concurrent downloading with a single keystroke.
- **Resilient Downloads:** Built-in support for download resumes. If a download is interrupted or fails, it picks up right where it left off.
- **Auto-Subtitle Fetching:** Automatically downloads the best-matching `.srt` subtitle files alongside your video files.

### Terminal Experience
- **Native Image Rendering:** Enjoy high-resolution movie posters rendered directly in supported terminals.
- **Dynamic Theming:** Switch between beautiful 3D block layouts and clean ASCII themes to fit your aesthetic.
- **Power-User Slash Commands:** Use terminal-style commands to update the app (`/update`), switch categories, or customize your Live TV playlists (`/config`).
- **Smart Auto-Cleanup:** A silent background worker intelligently manages and deletes old cache files to protect your disk space.


## Installation

**Prerequisites:** You will need a terminal (at least 85×24 characters) and a local video player installed (e.g. `mpv`, `IINA`, or `VLC`).

The easiest way to get started is by using our quick install scripts. These scripts will automatically download the correct version for your computer.

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.sh | bash
```

### Windows
```powershell
powershell -c "irm https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.ps1 | iex"
```

### Cargo (For Rust Developers)
```bash
cargo install moviebox-tui
```


## Getting Started

Once installed, just open your terminal and type `moviebox-tui` to jump in!

### Keyboard Controls

| Key | Action |
| --- | --- |
| Alphanumeric | Start searching instantly |
| <kbd>↑</kbd> <kbd>↓</kbd> <kbd>←</kbd> <kbd>→</kbd> | Navigate menus and grids |
| <kbd>Enter</kbd> | View details, pick episodes, or play video |
| <kbd>o</kbd> | Switch to a different video player on playback |
| <kbd>d</kbd> | Download an episode or an entire season |
| <kbd>Ctrl</kbd>+<kbd>p</kbd> | Switch between different content providers / sources |
| <kbd>Ctrl</kbd>+<kbd>t</kbd> | Toggle Live TV mode to browse IPTV channels |
| <kbd>?</kbd> | Open the global help menu |
| <kbd>q</kbd> | Quit (or use <kbd>Esc</kbd> to go back/clear search) |

### Slash Commands
You can type these special commands straight into the search bar:

| Command | Category | Description |
| --- | --- | --- |
| `/discover` or `/home` | Streaming | See what's trending right now |
| `/movies`, `/shows`, `/anime`| Streaming | Jump straight to a specific category |
| `/list` | Live TV | Show the list of available live channels |
| `/config` | Live TV | Open the TV configuration menu to add your own m3u playlists |
| `/update` | General | Check to see if there's a new version of the app |
| `/toggle-update` | General | Turn automatic background update checking on or off |


## Contributing

I'd love your help making this even better! If you've got a big feature in mind, it's usually best to open an issue first so we can chat about it.

```bash
git clone https://github.com/mesamirh/MovieBox-Tui.git
cd MovieBox-Tui
cargo build
```

Just try to follow [Conventional Commits](https://www.conventionalcommits.org/) and make sure `cargo fmt` and `cargo clippy` are happy before you open a PR. You can check out [CONTRIBUTING.md](CONTRIBUTING.md) for the full rundown.


## Credits & Legal

Live TV channel playlists are graciously provided by [iptv-org/iptv](https://github.com/iptv-org/iptv).

> **Disclaimer:** This is a third-party client. It does not host or store any media and only resolves links from upstream APIs. Intended for personal use only.


## Community & Support

The best way to support MovieBox-TUI is simply to use it, share it, and leave a star on GitHub!

If you'd like to buy me a coffee for the late nights spent coding, you can use the addresses below.

- **EVM:** `0x7ea20d5fa29d87f33195f5a3b211ff94038d794c`
- **BTC:** `3MEAtqtRWrQBhnaMi3Zuf5nt2efNUS2LUQ`
- **LTC:** `ltc1qhjkq2n6tsayxj56n3c53uqv23v8vqhvc9g3vxl`

---

<div align="center">

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.<br>
Built by [**@mesamirh**](https://github.com/mesamirh)

<sub>Not affiliated with any third-party content providers or operators.</sub>

</div>
