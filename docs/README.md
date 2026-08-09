# MovieBox-Tui Documentation

This folder documents the project so that anyone — human or AI — can understand how it
works without reading the whole codebase. Each document maps concepts to `file:line`
anchors that stay accurate as the code evolves.

## Index

| Document | What it covers | Status |
|---|---|---|
| [architecture.md](architecture.md) | Crate/module map, event loop, async model, data flow | current |
| [modules.md](modules.md) | Crate/module tree with responsibilities | pending (lands with the app.rs split) |
| [providers.md](providers.md) | MovieBox, 4KHDHub, BDIX protocols, signing, resolvers, errors | pending |
| [players.md](players.md) | Player detection, mpv/VLC/IINA/AndroidIntent, headers, subtitles, window sizing | pending |
| [cache.md](cache.md) | Cache layout, namespaces, TTLs, atomic writes, purge | pending |
| [logging.md](logging.md) | File logging, `MOVIEBOX_LOG`, rotation, sanitization, sharing logs | pending |
| [tv-mode.md](tv-mode.md) | User-owned M3U playlists, manager, search, playback, config | pending |
| [config.md](config.md) | `config.json` fields and `MOVIEBOX_*` environment variables | pending |
| [downloads.md](downloads.md) | Download engine: resume, ranges, segments, retry, cancel | pending |
| [cross-platform.md](cross-platform.md) | OS support, terminal protocols, Termux, focus handling | pending |
| [debugging.md](debugging.md) | Reproducing issues and what to include in GitHub reports | pending |
| [contributing.md](contributing.md) | Build, lint gate, commit conventions | pending |
| [known-issues.md](known-issues.md) | Known limitations and how they are tracked | pending |

> Docs marked "pending" are written incrementally as the corresponding refactor lands, so they
> always describe the current code rather than a planned state.
