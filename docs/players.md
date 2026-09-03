# Players

Playback is handed to an external player. `player.rs` detects available players and
builds the exact command; `tui/app/playback.rs` spawns it.

## Detection

`player::detect()` (powered by a centralized `probe_player_executable` engine) returns players in priority order, resolving and caching paths statically:

- macOS: IINA (if present), then mpv, then VLC (probing `/Applications`, `~/Applications`, Homebrew `/opt/homebrew/bin`, MacPorts `/opt/local/bin`, and standard `/bin`).
- Linux: mpv, then VLC (probing Native `$PATH`, Flathub/Flatpak user & system exports `org.videolan.VLC` / `io.mpv.Mpv`, Snap `/snap/bin/*`, standard `/bin`, and `flatpak run`).
- Windows: mpv, then VLC (probing `Program Files` including WinGet `MPV Player` and `mpv-player`, `LOCALAPPDATA\Programs`, `WinGet\Links`, portable `C:\mpv`, Scoop `scoop\shims`, and Chocolatey).
- Android/Termux: `mpv` (if installed via `pkg install mpv`), or Android intent chooser (`termux-open`, `termux-open-url`, `termux-am`, or `/system/bin/am`).

Resolution runs once at startup and is cached (`OnceLock`). A preferred player can be
forced via `MOVIEBOX_PLAYER` env or `default_player` in config (e.g. `mpv`, `iina`,
`vlc`, `android`), which reorders the list. The media player picker in the Settings Hub (`/settings`) lists every detected player on your system and saves your selection to `config.json` unless overridden by `MOVIEBOX_PLAYER`. Playback launches directly using the preferred compatible player without intermediate modal dialogs.

## Command construction

| Player          | Invocation                                                                                                                                | Notes                                                                                                  |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| mpv             | `mpv --autofit=WxH --geometry=50%:50% --idle=no --keep-open=no [--start=..] [--script=..] [--script-opts=..] [--http-header-fields=..] [--sub-file=..] <url>` | Window sized to the terminal. Injects `moviebox_tracker.lua` for position tracking and resume. Flatpak mpv is launched via `flatpak run`. |
| VLC             | `vlc --width=W --height=H --play-and-exit [--start-time=..] [--http-referrer=..] [--http-user-agent=..] [--sub-file=..] <url>`            | Supports start time resume via `--start-time`.                                                         |
| IINA            | `iina-cli --keep-running --no-stdin --mpv-autofit=.. [--mpv-start=..] --mpv-http-header-fields=.. --mpv-sub-files=.. <url>`               | Uses the installed IINA `iina-cli`; falls back to `open -a IINA <url>` only if the CLI is absent.      |
| Android / Termux | `termux-open --chooser --content-type video/* <url>` (or `termux-open-url` / `termux-am`) | Opens an app chooser on the device. Requires `termux-tools` (or `termux-am`). |

Window size is derived from the live terminal size times the font cell size reported
by the image picker, then clamped to a fixed range.

## Headers

Playback sources (for example 4KHD and MovieBox) may carry `Referer`/`User-Agent` headers.
MovieBox DASH streams additionally carry signed `Cookie` headers for CloudFront authentication.
mpv/IINA send them via `http-header-fields` (`--http-header-fields=...` or `--mpv-http-header-fields=...`),
while VLC maps them to `--http-referrer` / `--http-user-agent`.
The `supports_headers` gate in `app/playback.rs` validates whether a player can satisfy required stream headers. MovieBox-TUI strictly respects the user's configured default player: if the chosen player cannot satisfy a stream's headers (such as VLC attempting to play signed MovieBox DASH manifests with cookie requirements), playback will not silently fall back to an alternative player. Instead, an explicit warning notification informs the user of the exact incompatibility and lists available compatible alternatives (e.g. mpv) or the option to switch providers with `Ctrl+P`.
## Subtitles

- mpv receives the remote subtitle URL directly (`--sub-file=<url>`); mpv fetches it
  with the stream headers applied.
- VLC and IINA download the subtitle to a temp file first, preserving the URL's
  extension (srt/vtt/ass/…), and pass the local path. The download applies the source
  headers. On failure a status is shown and playback continues without subtitles.
- Temp files are cleaned up after the player exits and purged at startup if stale.

## Playback Tracking & Resume

When launching media with in-progress watch history, the player command automatically
includes the starting position (`--start` / `--mpv-start` / `--start-time`). For `mpv`,
`src/player/tracker.rs` writes a companion tracker script (`moviebox_tracker.lua`) that
observes `time-pos` and `duration`, periodically saving playback state every 5 seconds to
the local data directory. On startup or after player exit, pending playback states are
reconciled into `history.json`.

## Spawning

`launch_player` spawns the player with null stdin/stdout, piped stderr, and its own
process group (Unix) or no-console flag (Windows). A blocking task reads stderr and
reports every non-zero process exit as a player error, including failures with no
diagnostic output. Watch progress is reconciled only after a successful exit.
