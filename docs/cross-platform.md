# Cross-platform

Supported: macOS, Linux, Windows, and Android (Termux). The codebase uses
`crossterm` (all platforms), `ratatui`, and a couple of small per-OS branches.

## OS notes

| Platform         | Notes                                                                                                                                                                                                                                                                                                |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS            | IINA preferred player (via bundled `iina-cli`); VLC/mpv `.app` paths detected. `process_group(0)` on spawn.                                                                                                                                                                                          |
| Windows          | mpv/VLC detected via Program Files + `%LOCALAPPDATA%`; players spawned with `CREATE_NO_WINDOW`; path-safe file stems.                                                                                                                                                                                |
| Linux            | Flatpak mpv/VLC supported (`flatpak run …`); xdg data/cache dirs.                                                                                                                                                                                                                                    |
| Android (Termux) | Dynamic filesystem checks for `/system/bin/am` or `termux-open`; playback strips `LD_LIBRARY_PATH` and `LD_PRELOAD` before launching the chooser. Real-device chooser behavior should be confirmed separately because the release pipeline is desktop-focused; Proot/Termux X11 behavior is environment-dependent. |

## Terminal capabilities

The app probes the terminal at startup via `ratatui_image`:

- **Poster rendering**: High-resolution graphics via Sixel, Kitty, and iTerm2 protocols when supported by the terminal. Non-graphics terminals (e.g. standard Apple Terminal.app, basic xterm) gracefully display clean, aligned text placeholders (`Poster unavailable` / `No Art`) with zero block redraw artifacts. Image queries can be disabled via `MOVIEBOX_NO_IMAGE=1`.
- **Terminal classification**: `TERM=dumb`/`linux` fall back to a basic UI.
- Focus events (focus loss/gain) are used to re-render in place without clearing.

## Network & TLS portability

- **TLS Engine**: Uses pure-Rust `rustls` across all targets (macOS, Linux, Windows, Android/Termux), eliminating external C dependencies on OpenSSL.
- **DNS Resolution**: Uses standard POSIX/WinSock system DNS (`getaddrinfo`) on all platforms, ensuring reliable domain resolution in both standard OS environments and native Android/Termux CLI sessions without requiring JNI or `ndk-context`.

## Things to verify per release

- Player launch on each OS (mpv/VLC/IINA/Android intent) — window sizing, subtitles,
  headers.
- Poster rendering across Sixel (Windows Terminal, foot), Kitty, and half-block
  terminals.
- TV mode with a sample M3U playlist (URL and local file).
- Termux: on-device check that Play opens the Android chooser and the stream plays.
