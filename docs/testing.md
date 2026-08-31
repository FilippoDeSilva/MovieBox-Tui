# Testing & QA Architecture

This document describes the testing architecture, quality assurance procedures, and validation guidelines for MovieBox-TUI.

## 1. Test Architecture

The test suite comprises **296+ automated tests across 19 test suites** (158 unit tests in `src/lib.rs`, 0 binary tests in `src/main.rs`, and 138 integration tests across 18 test suites in `tests/`), running fully offline by default without mocking or live network dependencies.

The MovieBox-TUI test architecture follows a strict separation of concerns:

```text
MovieBox-TUI/
├── src/
│   └── **/*.rs              # Pure algorithms & inline unit tests (#[cfg(test)])
└── tests/
    ├── common/
    │   └── mod.rs           # Shared test utilities & temporary directory helpers
    ├── fixtures/
    │   ├── m3u/
    │   │   └── sample_playlist.m3u
    │   └── addons/
    │       └── manifest.json
    ├── settings_hub.rs        # Interactive Settings Hub modal tab navigation, choices, & option rows
    ├── history_reconciliation.rs  # History identity & background state reconciliation
    ├── history_audit.rs           # Cross-mode watch progress, series advancement, & boundary audit
    ├── grand_user_journey.rs      # End-to-end full multi-phase user lifecycle verification
    ├── cache_lifecycle.rs         # Disk cache hashing & atomic writes
    ├── player_integration.rs      # Cross-platform player command & path arguments
    ├── m3u_integration.rs         # Offline M3U playlist parsing
    ├── download_integration.rs    # File stem sanitization & folder structure
    ├── url_security.rs            # URL validation & device stem protection
    ├── error_handling.rs          # Failure state cleanup, error toasts, and recovery
    ├── content_pipeline.rs        # Content/metadata pipeline, stale request isolation, & cache keys
    ├── tui_acceptance.rs          # Headless TUI rendering, theme rendering, and resize matrix
    ├── update_lifecycle.rs        # Update single-flight checks, modal hitboxes, & platform assets
    ├── real_acceptance.rs         # Opt-in live artifact downloads, SHA256 integrity, & version verify
    ├── version_upgrade_e2e.rs     # Offline (mock-server) end-to-end upgrade execution
    ├── addons_manifest.rs         # Addon manifest deserialization & catalog checks
    ├── favorites_lifecycle.rs     # Favorites persistence, identity, and navigation lifecycle
    └── live_stream_verification.rs # Live MovieBox CDN signed stream & resolution verification (opt-in)
```

### A. Inline Unit Tests (`src/**/*.rs`)
Inline unit tests live inside `#[cfg(test)] mod tests` blocks within their respective modules. They verify:
- Title normalization (`clean_moviebox_title`)
- HMAC-MD5 cryptographic signing and token generation (`generate_x_client_token`, `generate_x_tr_signature`)
- Segment partitioning math and byte range calculations
- Internal helper logic, parsing functions, and release asset matching
- Update modal layout geometry calculations and semver comparisons

### B. Subsystem Integration Tests (`tests/*.rs`)
Integration tests live in the `tests/` directory and test externally observable behaviors without mocking internal types:
- **`version_upgrade_e2e.rs`**: Validates the full update pipeline against a local mock HTTP server (no real network), verifying discovery, asset and checksum download, SHA-256 integrity check, staging, atomic replacement, rollback, and verification that the newly installed binary reports its new version (Unix only).
- **`update_lifecycle.rs`**: Validates update single-flight concurrency barriers, error recovery on network failure, 1:1 mouse hit-test synchronization with rendered popup geometry, deterministic platform asset filtering, checksum integrity verification (valid, mismatch, missing, multi-format), archive extraction with strict path traversal rejection (`..`), active work protection (playback and download guards), environment detection (Homebrew, ReadOnly, DirectReplace), and safe atomic binary replacement/rollback.
- **`real_acceptance.rs`**: Validates real-world GitHub release artifact downloads, live streaming SHA-256 verification against upstream `SHA256SUMS`, actual archive extraction to non-empty executable binaries (`chmod 0755`), real `--version` validation of extracted binaries, Windows helper script batch syntax, and data preservation across updates. The live-download test is `#[ignore]`-gated so default runs stay offline; opt in with `MOVIEBOX_LIVE_TESTS=1 cargo test --test real_acceptance -- --ignored`.
- **`content_pipeline.rs`**: Validates search result identity, ambiguous title isolation, stale metadata response protection (`request_id` validation), cache key dimensional isolation, addon metadata mapping & partial degradation, search failure vs empty result status distinction, and mode-switch stale response isolation.
- **`error_handling.rs`**: Validates active player session lifecycle, playback debounce guards, search failure cleanup, addon manifest error toasts, stream and download resolution failure notifications, malformed M3U recovery, and URL scheme rejection.
- **`tui_acceptance.rs`**: Validates headless TUI rendering, all theme palettes, end-to-end user journeys (search, details, navigation, mode switching), mouse click and scroll interactions, modal dismissals, and terminal resize matrices across 8 standard and boundary dimensions without panics.
- **`history_reconciliation.rs`**: Validates `HistoryManager::is_same_show`, media type separation (Movies vs TV Series), remake year distinction, and state file reconciliation (MISS-01).
- **`history_audit.rs`**: Validates cross-mode watch progress, series advancement and completion tracking, threshold boundaries for in-progress states, history disk persistence roundtrips, Lua tracker reconciliation, and `/history` search list integration.
- **`favorites_lifecycle.rs`**: Validates Favorites persistence boundaries, identity deduplication, `/favorites` loading, landing-row navigation, and independence from watch-history clearing.
- **`cache_lifecycle.rs`**: Validates atomic file writing (both sync and async) and deterministic MD5 cache key generation.
- **`player_integration.rs`**: Validates MPV script options path sanitization on Windows (`\` $\rightarrow$ `/`) and Unix.
- **`m3u_integration.rs`**: Validates parsing of standard, single-quoted, double-quoted, and unquoted M3U playlists using fixtures.
- **`addons_manifest.rs`**: Validates deserialization of Cinemeta/Stremio addon manifests, series vs movie metadata classification, multi-season episode decomposition, episode stream isolation (`parse_season_episode`), token and codec parsing, and movie stream regressions.
- **`url_security.rs`**: Validates HTTP/HTTPS URL detection and Windows reserved device stem sanitization (`CON`, `AUX`, `PRN`, `NUL`, `COM1-9`, `LPT1-9`).
- **`live_stream_verification.rs`**: Validates real-world MovieBox CDN stream link resolution for movies and TV series, verifying that the backend returns valid signed MP4 streams and that legacy upgrade notice hashes (`1c7de0bd...`) are rejected. The tests are `#[ignore]`-gated for offline execution; opt in with `cargo test --test live_stream_verification -- --ignored`.

---

## 2. Running Automated Tests

Run the full test suite (all unit and integration tests):

```bash
cargo test --all-features --locked
```

Run only unit tests:

```bash
cargo test --lib --all-features --locked
```

Run a specific integration test:

```bash
cargo test --test history_reconciliation --all-features --locked
cargo test --test player_integration --all-features --locked
cargo test --test m3u_integration --all-features --locked
```

Run the opt-in live-network acceptance test:

```bash
MOVIEBOX_LIVE_TESTS=1 cargo test --test real_acceptance --all-features --locked -- --ignored
```

---

## 3. Code Hygiene and Static Analysis

Every commit and pull request must pass all hygiene checks:

```bash
# Check formatting
cargo fmt -- --check

# Compiler check
cargo check --all-targets --all-features --locked

# Clippy linter with warnings treated as errors
cargo clippy --all-targets --all-features --locked -- -D warnings

# Security vulnerability scan
cargo audit
```

---

## 4. Manual QA Matrix

Because TUI and media player interactions depend on terminal capabilities and external processes, manual QA testing covers:

### Terminal Emulators
- **Ghostty** (Primary testing terminal)
- **macOS Terminal.app**
- **iTerm2**
- **Windows Terminal**
- **Linux VTE / Alacritty / Kitty**
- **tmux** session & pane switching

### Supported Media Players
- **mpv** (Full IPC tracking, script options, state reconciliation)
- **VLC** (Local playback and stream forwarding)
- **IINA** (macOS native player integration)
- **termux-open / Android Intents** (Android Termux)

### Key Flows to Verify
1. **Search & Browse**: Type queries, switch between MovieBox, 4KHDHub, BDIX, Addons, and Live TV.
2. **Playback Launch & Return**: Launch stream in player, exit player, verify terminal state is cleanly restored without residual escape sequences.
3. **History & State**: Verify playback progress and watched checkmarks update accurately.
4. **Downloads**: Test single episode and batch season download queuing.
