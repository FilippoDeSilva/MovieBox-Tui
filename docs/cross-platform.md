# Cross-platform

Build targets: macOS, Linux, Windows, and Android (Termux via the Linux ARM64
binary). The codebase uses
`crossterm` (all platforms), `ratatui`, and a couple of small per-OS branches.

"Build target" does not mean every runtime integration is verified on every
device. Desktop builds are covered by CI; external players and Termux require
the release checks in [`release-checklist.md`](release-checklist.md).

## OS notes

| Platform         | Notes                                                                                                                                                                                                                                                                                                |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS            | IINA preferred player (via the installed IINA `iina-cli`); VLC/mpv `.app` paths detected. `process_group(0)` on spawn.                                                                                                                                                                                |
| Windows          | mpv/VLC detected via Program Files + `%LOCALAPPDATA%`; path-safe file stems; static MSVC CRT linking (`+crt-static`) for zero-dependency standalone binaries. |
| Linux            | Flatpak mpv/VLC supported (`flatpak run …`); xdg data/cache dirs.                                                                                                                                                                                                                                    |
| Android (Termux) | Uses the Linux ARM64 binary compiled as static-PIE (`ET_DYN`) with 64-byte `PT_TLS` alignment to satisfy Android Bionic (`/system/bin/linker64`) and kernel W^X execution rules without `unexpected e_type: 2` errors. Includes a pure-Rust DNS resolver that reads system configuration and falls back to public resolvers (Cloudflare, Google, Quad9) for zero-config name resolution, and dynamically checks for `termux-open`, `termux-open-url`, `termux-am`, or in-terminal `mpv`; playback strips `LD_LIBRARY_PATH` and `LD_PRELOAD` before launching the chooser. Android intent playback cannot forward stream headers or attach subtitles. Real-device chooser behavior is a release prerequisite; Proot/Termux X11 behavior is unsupported unless separately tested. |

## Terminal capabilities

The app probes the terminal at startup via `ratatui_image` (400ms cap,
off the UI thread). Non-graphics terminals (e.g. macOS `Apple_Terminal`,
legacy Windows `conhost`, and `TERM=dumb/linux/cygwin`) are skipped to
prevent escape sequence probe leakage (`Gi=31...`):

- **Poster rendering**: Sixel, Kitty, and iTerm2 protocols where the probe
  detects them. Terminals that report kitty/sixel capability but no cell
  size (Windows Terminal sixel, iTerm2 over SSH) are salvaged with default
  cell metrics. True Halfblocks-only terminals show a compact filmstrip
  placeholder instead. All poster image rendering is automatically gated
  behind `has_active_modal()` to prevent image protocol graphic bleed
  through modal popups and overlays. `MOVIEBOX_NO_IMAGE=1` disables queries;
  `MOVIEBOX_IMAGE_PROTOCOL` forces a protocol; `MOVIEBOX_CELL_SIZE=WxH`
  overrides metrics.
- **Colors & Themes**: With no explicit theme, `NO_COLOR` wins, then truecolor RGB
  (auto-detected across Ghostty, Kitty, WezTerm, iTerm2, Alacritty, Foot, Windows Terminal,
  Hyper, Tabby, Warp, and VSCode), quantized 256-color palettes for strict terminals,
  and a 16-color ANSI fallback palette (`Theme::fallback`); an OSC 11 background query
  picks light/dark variants. Light mode themes (including Catppuccin Latte) are tuned for
  WCAG AA compliance, ensuring high-contrast readability across light terminal backgrounds.
  An explicit `MOVIEBOX_THEME` or saved theme always wins over autodetection.
- **Keyboard & Cursor**: The kitty keyboard protocol (disambiguated escapes, event
  types) is requested at start and popped on exit; unsupported terminals
  ignore it. Real input cursor styling (`SetCursorStyle::SteadyBar`) activates
  during text editing modes on supported terminals.
- **Loading & Progress Animations**: Fluid 10-frame Unicode Braille loading
  spinners (`⠋ ⠙ ⠹ ...`) render during search, metadata discovery, and stream
  fetching, falling back to clean text indicators (`..`) on basic/dumb terminals.
- **Window Titles**: Contextual terminal emulator window titles are emitted dynamically
  reflecting active navigation mode and title (`MovieBox-Tui — Streaming`,
  `MovieBox-Tui — Live TV`, `MovieBox-Tui — Addons`, `MovieBox-Tui — {Title}`).
- **Terminal classification**: `TERM=dumb`/`linux` fall back to a basic UI.
- Focus events re-render in place without clearing; results render in two
  columns from 110 columns wide (three from 160).
- **Responsive Installers**: Both Unix (`install.sh`) and Windows (`install.ps1`) installation scripts query terminal width dynamically (`tput cols` / `stty size` / `$Host.UI.RawUI.WindowSize.Width`), adapting the header across wide (72-column block art), compact (31-column 2-line half-block art for mobile/Termux portrait mode), and minimal (text-only) tiers with dynamic horizontal centering to prevent line wrapping.
## Network & TLS portability

- **TLS Engine**: Uses `rustls` with embedded Mozilla roots (`webpki-roots`) across all targets (macOS, Linux, Windows, Android/Termux). The release binary has no OpenSSL runtime dependency; the `ring` cryptography backend is compiled into the binary by the platform build toolchain.
- **DNS Resolution**: Pure-Rust resolver (hickory) on all platforms: it reads the OS
  configuration first (`/etc/resolv.conf`, registry on Windows) and falls back to
  embedded public resolvers (Cloudflare `1.1.1.1`, Google `8.8.8.8`, Quad9 `9.9.9.9`)
  when no system configuration exists — as on Android/Termux or minimal containers.
  No JNI or `ndk-context` required.

## Things to verify per release

- Player launch on each OS (mpv/VLC/IINA/Android intent) — window sizing, subtitles,
  headers.
- Poster rendering across Sixel (Windows Terminal, foot), Kitty, iTerm2, and basic
  non-graphics terminals.
- TV mode with a sample M3U playlist (URL and local file).
- Addon Mode with sample HTTP addon manifests (Cinemeta, torrent/stream addons).
- Termux: on-device check that Play opens the Android chooser and the stream plays.
