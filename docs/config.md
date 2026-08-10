# Configuration

## `config.json`

Written atomically under the config directory (`dirs::config_dir()/moviebox-tui/`;
macOS `~/Library/Application Support/moviebox-tui`, Linux `~/.config/moviebox-tui`).

| Field | Type | Meaning |
|---|---|---|
| `auto_update` | bool | Check for updates on startup (max once/hour). |
| `last_update_check` | u64 | Epoch seconds of the last update check. |
| `active_provider` | string | Last provider (`moviebox`, `fourkhdhub`, …). |
| `active_theme` | string | Theme name. |
| `bdix_enabled` | bool | Show BDIX providers (Bangladesh-only). |
| `default_player` | string | Preferred player: `mpv`, `iina`, `vlc`, `android`. |

## Other persisted files

- `tv_config.json` — list of M3U playlist sources (see [tv-mode.md](tv-mode.md)).
- `history.json` — watch history.
- `iptv_cache/` — legacy TV image cache directory that `ClearCache` still removes.

## Environment variables

| Variable | Purpose |
|---|---|
| `MOVIEBOX_LOG` | Log level: `off`, `warn`, `info`, `debug`, `trace`. See [logging.md](logging.md). |
| `MOVIEBOX_PLAYER` | Preferred player (overrides `default_player`). |
| `MOVIEBOX_MPV_PATH` | Custom mpv executable. |
| `MOVIEBOX_VLC_PATH` | Custom VLC executable. |
| `MOVIEBOX_IINA_PATH` | Custom IINA/iina-cli executable. |
| `MOVIEBOX_FOURKHDHUB_URL` | Override the 4KHDHub base URL. |
| `MOVIEBOX_THEME` | Force a theme (e.g. `Mocha`, `Latte`). |

## CLI

- `moviebox-tui --version` prints the version and exits.
- No other flags.
