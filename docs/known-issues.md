# Known issues and limitations

Tracked here so future work and issue reports reference the same facts.

## Latent / by-design

- **`supports_headers` gate is currently never triggered** — every `PlaybackSource`
  today either carries no headers or only `referer`/`user-agent`. If a provider ever
  adds other headers, the VLC path would reject them (by design) and the gate message
  would matter. Keep `player.rs::supports_headers` in sync with `vlc_command`'s filter.
- **BDIX clients use nested `if let Ok` pyramids** in search handling; they work and
  are logged, but are harder to read. Flattening is deferred (behavior-neutral refactor
  with moderate churn).
- **MovieBox request signing** hardcodes an API secret and spoofs a device identity in
  `crypto.rs`. This is inherent to the scraper; treat the module as one unit.
- **Android intent playback** cannot attach subtitles (a `VIEW` intent has no subtitle
  mechanism); subtitles are ignored for the Android player.

## Environment-dependent

- **4KHDHub mirrors rotate and can be region/rate limited.** A file whose only mirrors
  are "probe trap" workers (which refuse real streaming ranges) reports
  `no playable mirrors` with the reason in the log. Not fixable in-app.
- **Termux playback needs the device confirmed** on each release: `termux-open` /
  `am` availability and the Android chooser behavior.

## Test coverage

- There are no unit tests currently (per project decision); correctness is enforced by
  the lint gate, type checking, and manual verification. CI runs `cargo audit` and
  `cargo package`.
