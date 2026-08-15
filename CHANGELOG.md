# Changelog

## [Unreleased]

### Added
- **CLI Help Flag**: Added `-h` / `--help` CLI flags printing formatted usage, available options, and environment variables.
- **Full Mouse Support**: Complete mouse navigation throughout the application:
  - Click search bar to edit; click suggestion items to search immediately.
  - Click search results to select/preview; click again or double click to enter Details.
  - Click Details panes (Audio Languages, Seasons, Episodes, and Streams) to select and launch playback.
  - Click centered footer toolbar buttons (`[Ctrl+P] Provider`, `[Ctrl+T] TV`, `[?] Help`, `[q] Quit`).
  - Full click support across all modal popups (Theme, Browse, Subtitles, Players, TV playlists & actions, and Download confirmation).
- **Contextual Downloads**:
  - Pressing `d` or clicking `[Download]` while on the **Seasons** pane prompts to download the whole season (all episodes).
  - Triggering download while on **Episodes** or **Streams** downloads that single episode.
- **Organized Downloads & Custom Directory**:
  - Structured Series downloads under `<base_dir>/Series/<Title>/Season <N>/<Title> - S<N:02>E<E:02>.<ext>` (Plex/Jellyfin standard) and Movies under `<base_dir>/Movies/<Title>/<Title>.<ext>`.
  - Added ISO 639-1 language code tagging to subtitle sidecars (e.g. `<BaseName>.en.srt`) for automatic track identification in media players and servers.
  - Added smart duplication prevention: completed episodes on disk are automatically skipped during season batch downloads.
  - Added `/download-dir <path>` slash command with directory creation and active write-probe validation.
  - Added `/download-dir reset` (contextually suggested only when custom path is configured) to revert to OS default.
  - Safe automatic fallback to default OS Downloads folder if custom path becomes inaccessible.
  - Configuration persistence across sessions in `config.json`.
- **Tree Branch Suggestions**:
  - Redesigned search and slash command autocomplete into a minimal, transparent tree-branch layout (`├─ ` / `└─ `) anchored directly under the search prompt.
  - Added aligned slash command descriptions (`browse`, `history`, `theme`, `config`, `update`, etc.) without duplicate leading slashes.
  - Clean typography-driven active selection with bold vibrant accent styling.
- **Multilingual Audio Track Detection**:
  - Expanded 4kHDHub release parser to detect 30+ regional and international languages (Hindi, Tamil, Telugu, Kannada, Malayalam, Bengali, Marathi, Punjabi, Gujarati, Urdu, Japanese, Korean, Chinese, Spanish, French, German, Italian, etc.) and abbreviations (`Tam`, `Tel`, `Kan`, etc.).
  - Responsive stream list formatting showing all available languages without crowding mirror counts.
- **Floating Pill HUD & Smooth Resize**:
  - Added floating terminal dimension HUD and event coalescing for smooth window resizing without blank screens.
- **Elevated Notification Badges**:
  - Redesigned notification popups into elevated, rounded bottom-right badge cards with clean typography.
- **Persistent Long-Term Poster Caching**:
  - Increased image cache retention to 30 days (`IMAGE_CACHE_EXPIRY_SECS`), serving previously fetched posters instantly from disk across sessions with zero redundant network requests.
  - Unified image caching under a shared namespace with automatic cross-namespace lookup across MovieBox, 4KHDHub, IPTV, CircleFTP, and DhakaFlix.
- **Streamlined Browse Views**:
  - Curated `/browse` views into 4 categorized shelves (Popular, Top Rated, Trending, Most Watched) with proper filtering.
- **Terminal Graphics Fallback Overrides**:
  - Added `MOVIEBOX_NO_IMAGE=1` and `MOVIEBOX_HALFBLOCKS=1` environment overrides to disable terminal image querying or force Unicode halfblock graphics for SSH, tmux, and slow remote connections.

### Fixed
- **Default Audio Track Prioritization (Original / English)**:
  - Fixed movie and series details defaulting to regional Hindi dubs on MovieBox by prioritizing `Original` and `English` audio tracks over localized search result subject IDs.
  - Preserved explicit user language selections when intentionally switching between dubs.
- **Home Landing Header & Footer Persistence**:
  - Fixed ASCII logo header and shortcut footer disappearing into a blank screen when clearing history or viewing empty search states by removing fragile tick-based animation gates.
  - Ensured the landing screen renders the logo, version, centered search bar, and footer shortcuts immediately on every frame.
- **Watch History Consolidation & Latest Progress Representation**:
  - Consolidated watched episodes of the same series into a single entry per show in `/history` displaying the latest watched season and episode.
  - Automatically deduplicated and migrated legacy history rows on startup while maintaining complete per-episode checkmark indexes in `self.watched`.
- **History Poster Auto-Hydration & In-Memory Cache Retention**:
  - Fixed "Poster unavailable" placeholders in `/history` by automatically resolving missing cover URLs and decoding posters in the background.
  - Preserved in-memory decoded image caches when opening `/history` to eliminate unnecessary UI redraw latency.
  - Added multi-source fallback extraction for cover URLs across playback, preview, and search results.
- **Stream Pool Initialization on Audio Selection**: Fixed stream fetching hanging on "Loading streams..." when selecting non-default audio dubs by ensuring stream pool entries are initialized before episode fetch.
- **Title Sanitization & Preservation**: Enhanced `clean_moviebox_title` to sanitize international audio dubs, video quality tags, and format markers across downloads, folder organization, and watch history while preserving 4-digit release years.
- **Terminal Restoration & Signal Handling**: Added `Ctrl+C` keyboard handling and asynchronous `SIGINT` signal listener to guarantee raw mode and alternate screen are always cleanly restored.
- **Download Hierarchy & Numbering**: Fixed series media type detection and removed season/episode off-by-one addition.
- **Parser UTF-8 Safety**: Hardened language detection boundary checks for multibyte titles against panics.
- **Startup Screen Artifacts**: Removed early startup `eprintln!` to eliminate terminal screen artifacts before entering alternate screen mode.
- **Android / Termux Stability**: Removed `hickory-dns` from network dependencies to resolve NDK context panics and crashes on Android.
- **Screen Flickering & Blanking**:
  - Eliminated full terminal clear on list navigation and infinite scroll pagination.
  - Fixed screen blanking when pressing `Esc` or resizing windows.
  - Replaced terminal clear with direct backend clear to eliminate cursor read timeouts.
- **Search & Navigation**:
  - Fixed search bar auto-closing when switching providers.
  - Fixed provider switching delays and event stream drops.
  - Kept chosen audio dub selected and prevented unwanted pane jumping on details refresh.
  - Handled empty query loading states and preset failures gracefully.
- **Downloads & Playback**:
  - Resolved MovieBox movie stream key mismatches and hardened resilient download flows.
  - Protected active downloads from accidental cancellation when typing `x` in the search bar.
  - Fixed playback lock edge cases and subtitle picker clipping.
- **Theme & Configuration**:
  - Fixed theme cancellation reverting correctly without persisting unapplied themes.
  - Unified `/theme` command and removed obsolete `/discover`, `/tab`, and `/themes` aliases.

### Changed
- Removed startup screen delay for instant app launch.
- Modernized in-place update notifications and dialogs.
- Rendered details footer on a single clean line to balance bottom margins.
- Removed search bar underline clutter in favor of clean header spacing.

## [0.1.11] - 2026-08-11

### Added
- **User-Owned M3U Playlists**: Full custom playlist management in TV mode with remote URL and local file support.
- **Android Runtime Support**: Termux playback and shared-storage handling continue to be exercised on real devices, but release artifacts remain desktop-focused.

### Refactored
- **Domain Modularization**: Split the application monolith into cohesive domain modules (`network`, `playback`, `download`, `requests`, `navigation`, `tv`, `system`, `keyboard`).
- **Provider Architecture**: Introduced typed contract models and unified `Provider` / `ReleaseProvider` seams.

### Fixed
- Added request generation IDs to invalidate stale asynchronous network and preview responses.
- Hardened MovieBox HMAC-MD5 request signing and runtime token initialization.
- Preserved upstream subtitle sidecar file extensions (`.srt`, `.vtt`, `.ass`, `.sub`).

## [0.1.10] - 2026-08-06

### Added
- **BDIX Integration**: Added CircleFTP and DhakaFlix provider scrapers with isolated cache namespaces.
- **Native Android Support**: Added Android Intent media player dispatch and shared storage download directory detection.
- **BDIX Commands**: Added `/enable-bdix` and `/disable-bdix` slash commands with search autocomplete.

### Fixed
- Fixed subtitle loading in IINA player.
- Fixed help menu text truncation on small terminal viewports.

## [0.1.9] - 2026-08-05

### Added
- **Watch History**: Added persistent watch history tracking via `/history` with automatic deduplication.
- **Update Checker**: Added in-app update notification popup with release information.
- **Subtitle Caching**: Added caching layer for MovieBox stream subtitles.

### Fixed
- Updated 4kHDHub mirror resolution for new HubCloud domains and anti-bot challenges.
- Prevented playback resolution request spam on rapid navigation keystrokes.

## [0.1.8] - 2026-08-02

### Added
- **Theme Switcher**: Added interactive theme picker popup with configuration persistence in `config.json`.
- **Homebrew Support**: Created official Homebrew tap formula with automated release updates.
- **Flatpak Support**: Added automatic detection for Flatpak-installed mpv and VLC players on Linux.

### Performance
- Migrated TUI event loop to bounded asynchronous channels.
- Added disk caching for provider media details during list preview loading.

## [0.1.7] - 2026-07-31

### Added
- **4KHDHub Provider**: Added secondary 4K content provider with HubCloud mirror resolver and preflight validation.
- **Bi-directional Navigation**: Added `Tab` and `Shift+Tab` pane switching on the Details screen.
- **Stream Caching**: Added in-memory stream caching with atomic disk writes.
- **Terminal Capabilities**: Added Sixel graphics and adaptive terminal layout detection.

### Fixed
- Managed temporary subtitle file lifecycle with delayed cleanup after player process initialization.
- Centered poster images vertically on wide terminal viewports.

## [0.1.6] - 2026-07-26

### Added
- **Live TV (IPTV) Mode**: Added IPTV live channel streaming with M3U parser and category groupings.
- **Confirmation Dialogs**: Added confirmation modals for season and episode downloads.
- **Player Picker**: Added interactive media player picker popup (`o` key).
- **Linux ARM64**: Added native build and install support for aarch64 Linux systems.

### Fixed
- Enhanced download engine with HTTP range requests, timeout handling, and chunk validation.

## [0.1.5] - 2026-07-23

### Added
- Initial tagged release:
  - Search, browse, stream, and download movies, series, and anime.
  - External player integration for mpv, VLC, and IINA.
  - Subtitle track picker and automated downloading.
  - Basic terminal ASCII fallback mode.
