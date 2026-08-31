# Changelog

## [Unreleased]

### Added
- **Unified `f` Shortcut for Favorites (`src/tui/app/keyboard.rs`, `src/tui/screens/help.rs`)**: Standardized favorite / unfavorite toggling across all screens exclusively to `f` / `F`, removing redundant `*` keybindings and streamlining in-app help overlays and documentation.
- **Responsive Details Screen Layout & Narrow Terminal Polish (`src/tui/screens/details.rs`, `src/tui/app/mouse.rs`)**:
  - **Narrow-Screen Dynamic Selector Pane Focus**: On compact terminal widths (<85 columns), renders the active selector pane (Audio, Seasons, or Episodes) with 100% available horizontal width instead of horizontally cramming 3 unreadable 14-character columns that cut off titles, language names, and season numbers.
  - **Adaptive Pane Titles & Border Overflow Guard**: Dynamic title formatting for selector panes and streams headers (`"Streams · N available · X/Y"` $\to$ `"Streams · N (X/Y)"` $\to$ `"Streams (N)"`) preventing Ratatui border title truncations (`"● Audio · 4  1/"`).
  - **Compact Header & Responsive Poster Guard**: Automatically suppresses the side-by-side poster column when width is <75 columns, giving full terminal width to the title, IMDb badge, metadata, and synopsis without tight character clipping.
  - **Multi-Tier Responsive Stream Table Layout**: Added dedicated 4-tier column formatting across ultra-compact (<58 cols), compact (58..85 cols), standard (85..115 cols), and wide (>=115 cols) widths.
- **4KHDHub Fast Stream Resolution & Bounded Concurrency (`src/providers/fourkhdhub/`)**:
  - **Path Percent-Encoding Normalization (`hubcloud.rs`)**: Automatic normalization of raw stream URLs containing unencoded spaces, brackets, and special characters using `percent_encoding::utf8_percent_encode` with `NON_ALPHANUMERIC`, preventing `InvalidUrl` errors on complex release paths.
  - **Prioritized Mirror Scoring (`hubcloud.rs`)**: Refactored candidate stream scoring to prioritize direct Google Video/CDN backends (`score 0`) and PixelDrain API (`score 1`) ahead of storage CDNs (`score 2`) and Cloudflare workers (`score 3`).
  - **Concurrent Mirror Resolution & Bounded Probing (`client.rs`)**: Concurrently fetches resolver URLs across available release mirrors with a 4.0-second timeout, probing candidates in prioritized chunks of 3 via `futures::future::select_ok` for fast (<2s) playback start.
  - **Upstream HTML Error Detection (`client.rs`)**: Inspects preflight response bodies for expired file notices (`"Failed to extract link"`, `"Token Expired"`, `"File Not Found"`, `"404"`), rejecting dead torrent mirrors immediately instead of hanging on missing redirect query parameters.
  - **Actionable User Guidance & Reset Safety (`playback.rs`, `download.rs`, `system.rs`)**: Standardized resolution timeouts to 18s, formatted notifications with specific guidance (`"4KHDHub Stream Unavailable: Mirrors for this release are dead or expired on 4KHDHub. Select another release (e.g. 1080p) or press Ctrl+P for MovieBox."`), and enforced `is_resolving_playback` state reset on all error and cancellation paths.
- **Mode-Aware Dynamic Search Suggestions & Rotating Hints (`src/tui/screens/home.rs`)**: Mode-aware rotating search suggestion hints cycling at 4-second intervals across Streaming, Addon, and TV modes providing title recommendations, slash command discovery, and mode-specific tips with priority fallback for active status messages.
- **Smooth Precision Beam Cursor & Active Keystroke Debounce (`src/tui/screens/home.rs`)**: High-contrast vertical beam cursor (`▎` in `theme.accent`, `█` on basic terminals) with zero gap spacing against ghosted placeholder text, balanced 500ms idle blinking cadence, and active typing debounce preventing mid-keystroke cursor flickering.
- **Multi-`Esc` Search Results Return-to-Homepage Navigation (`src/tui/app/keyboard.rs`)**: Enhanced `Esc` navigation so pressing `Esc` from search results focuses the search input (`InputMode::Editing`), and pressing `Esc` again clears the search state and returns directly to the landing screen / homepage.
- **Core Domain Typing & Modernized Provider Traits (`src/models.rs`, `src/providers/`)**:
  - Refactored `Provider` and `ReleaseProvider` trait methods to return strongly typed domain models (`Result<Vec<CatalogItem>, ProviderError>`, `Result<MediaDetails, ProviderError>`, and `Result<Vec<Release>, ProviderError>`) instead of untyped `serde_json::Value` objects.
  - Introduced structured, classified error boundaries (`ProviderError`) with automatic user notification formatting via `.user_message(provider)` and `From` conversions for provider client errors (`ScraperError`, `FourKHdHubError`, `CircleFtpError`, `DhakaFlixError`).
  - Added strongly typed domain adapters (`moviebox_search_json_to_catalog`, `moviebox_details_json_to_media_details`, `meta_to_catalog_item`, `meta_detail_to_media_details`) and exposed `search_typed` and `details_typed` on `MovieBoxService`.
- **In-Flight Async Task Tracking & Cancellation (`RequestTaskHandles`)**:
  - Added `RequestTaskHandles` to `App` with atomic `JoinHandle::abort` cancellation triggers (`cancel_search`, `cancel_details`, `cancel_streams`, `cancel_suggest`, `cancel_homepage`, `cancel_all`), preventing stale network requests and CPU leaks when navigating or changing search queries.
- **AppState Domain Partitioning & Sub-States (`src/tui/state.rs`)**:
  - Partitioned the monolithic `AppState` into cohesive domain sub-states (`UiState`, `CatalogState`, `PlaybackState`, `DownloadState`) to decouple modal UI flags, search catalogs, playback handles, and download queues.
- **Unified Header-Aware List Navigation (`src/tui/state.rs`, `src/tui/app/keyboard.rs`)**:
  - Consolidated duplicate manager list wrapping, header-skipping, and step math into reusable helpers (`step_header_aware_list`, `step_tv_manager_selected`, `step_addon_manager_selected`, `first_*`, `last_*`).
- **Unified Screen Layout & Mouse Hitbox Geometry (`src/tui/screens/details.rs`, `src/tui/app/mouse.rs`)**:
  - Exported `DetailsScreenLayout` and `details_screen_layout` so both visual rendering and mouse hit-testing derive from identical calculated rect bounds across all viewport tiers.
- **Centralized Slash Command Dispatch & Complete Command Inventory (`src/tui/commands.rs`, `src/tui/app/search.rs`)**:
  - Centralized command input-clearing pre-conditions and expanded `SlashCommand::ALL` to all 19 registered variants.
- **Unified Interactive Settings & Preferences Hub (`/settings`)**: Introduced a consolidated, mouse-friendly modal dialog replacing fragmented slash commands. Features 4 categorized tabs (`General`, `Content Modes`, `Appearance`, `Maintenance`), installed media player selection via interactive popup picker, live visual theme palette swatch picker integration, inline download directory editing, and multi-mode toggling with safety guards. Streamlined slash command autocomplete suggestions and in-app help overlays while retaining full legacy command compatibility.
- **Details Screen Subtitles Shortcut (`[s]`)**: Added `s`/`S` keyboard shortcut on Details screen to open the subtitle language picker, matching mouse click and footer hints.
- **Theme picker 3-point color swatches**: Rendered 3-point color swatches (`■ ■ ■` Accent | Surface | Base) previewing palette colors in `/theme`.
- **Category origin pill badges**: Added `[MOVIES]`, `[SERIES]`, and `[DISCOVER]` category badges in the `/browse` preset dialog.
- **Provider origin tags and resolution badges**: Displayed provider tags (`[MovieBox]`, `[4KHD]`, `[CircleFTP]`, `[DhakaFlix]`, `[Addon]`) and resolution badges (`[1080p]`, `[4K]`) on Home search result cards.
- **Result position and pagination indicator**: Added contextual item counter and page indicators (`Item X of Y • Page N/M`) on search results.
- **Contextual Details footer action bar**: Context-aware footer action bar with pane-specific shortcuts, `[f] Favorite`, and `[s] Subtitles`.
- **Fixed-width, zero-padded episode items**: Aligned episode numbers (`EP 01`, `EP 02`, `EP 10`) with fixed-width watch state indicators and timestamps.
- **Centralized UI widgets subsystem (`src/tui/widgets`)**: Standardized single-line text input
  fields (`render_single_line_input`), viewport-accurate Ratatui scrollbars (`render_scrollbar`),
  modal dialog framing and footer action bars (`ModalFrame`, `render_modal_footer`), and unified
  media badge and codec tag extraction (`badge.rs`) across search, details, and modal popups.
- **Search suggestion source badges**: Added visual pill origin tags (`[CMD]`, `[HISTORY]`,
  `[FAVORITES]`, `[TV]`, `[SUGGEST]`) to dropdown suggestions for instant origin clarity.
- **Responsive single-column result cards on narrow terminals (<75 cols)**: Dynamic single-column
  grid fallback on compact terminal windows ensuring titles, badges, ratings, and tags never clip
  or distort horizontally.
- **Search result jump & pagination ergonomics (`Home`, `End`, `PageDown`, `PageUp`, `g`, `G`)**:
  In `Normal` mode on the search results screen, `Home` and `g` jump to the top result, `End` and `G`
  jump to the bottom result while automatically fetching the next page, and `PageDown`/`PageUp`
  page through results.
- **1-Key direct watch history resume (`Space` / `P`)**: Pressing `Space` or `P` on any item in
  `/history` immediately launches direct playback for the recorded season and episode without
  manual Details navigation. Opening Details from history (`Enter`) pre-seeds selection to the
  recorded season/episode.
- **Non-destructive `Esc` search navigation & `/clear`**: In `Normal` mode with active search results,
  pressing `Esc` focuses the search bar (`InputMode::Editing`) for quick query adjustments rather
  than wiping results; pressing `Esc` on an empty query or using `/clear` returns to landing cleanly.
- **Actionable zero-results & multi-line wrapped error cards**: Zero-results states now display
  interactive guidance shortcuts (`[Ctrl+P] Switch provider`, `[/browse] Browse categories`,
  `[Ctrl+U] Clear`), and error states wrap full diagnostic descriptions with `[r] Retry request`
  and `[Esc] Back` action pills.
- **Synopsis wrap line-clamping & Details fetch error fallback**: Synopsis paragraphs are now bounded
  by actual visual line capacity with `wrap_text` rather than raw character counts, and failed
  details fetches render an actionable error box (`[r] Retry fetch`, `[Esc] Back`) rather than
  hanging indefinitely on the loading spinner.
- **Grapheme-safe search & text input centralization (`TextInputBuffer`)**: Unified all text
  editing across the main Search Bar, TV Playlist Manager, and Addon Manager with full
  grapheme-cluster awareness, horizontal cursor navigation (`Left`/`Right`, `Home`/`End`),
  forward deletion (`Delete`), backward deletion (`Backspace`), backward word deletion (`Ctrl+W`),
  and whole-line clearing (`Ctrl+U`).
- **High-contrast media resolution badges & audio/codec tags**: Color-coded, high-contrast
  media resolution badges (`4K UHD`, `1080p FHD`, `720p HD`, `SD`) and granular audio/video
  codec tags (`HDR`, `DV`, `ATMOS`, `5.1`, `HEVC`, `AV1`, `BluRay`, `WEB-DL`, `REMUX`) in Details
  stream listings, with theme-aware foreground/background contrast styling and clean bracketed
  ASCII fallbacks on basic terminals.
- **Smooth 10-frame Unicode Braille loading spinners**: Added fluid 10-frame Braille animation
  spinners (`⠋`, `⠙`, `⠹`, `⠸`, `⠼`, `⠴`, `⠦`, `⠧`, `⠇`, `⠏`) for search queries, metadata discovery,
  and stream fetching states, with graceful 2-character ASCII fallback (`..`) on basic terminals.
- **Horizontal Details pane navigation (`Left`/`Right`/`h`/`l`)**: Added intuitive horizontal
  directional navigation (`Left`/`Right` and `h`/`l` vim keys) to smoothly cycle between Audio
  Languages, Seasons, Episodes, and Streams panes in the Details view alongside `Tab` and `Shift+Tab`.
- **Surfaced backdrop container for slash command suggestions**: Autocomplete and slash command
  suggestions now render inside a dedicated `theme.surface0` background container with clear
  tree-branch hierarchy glyphs (`├─`, `└─` with `|-`, `\-` fallbacks) and aligned descriptions.
- **Toast notification clearance & visual countdown progress bar**: Elevated toast notifications
  dynamically adjust their bottom clearance (`bottom_offset: 5`) when the active download gauge
  is visible to prevent visual collisions, and feature a live countdown progress bar (`━───` and
  `[==--]`) indicating remaining toast lifespan.
- **Contextual terminal window titles**: Dynamically updates the terminal emulator window title
  based on active navigation state (`MovieBox-Tui — Streaming`, `MovieBox-Tui — Live TV`,
  `MovieBox-Tui — Addons`, and `MovieBox-Tui — {Title}` on Details screens).
- **Modal graphic anti-bleed gating (`has_active_modal()`)**: Suppresses background Kitty, Sixel,
  and iTerm2 poster rendering whenever any modal overlay or dialog is active (Help, Theme picker,
  Browse categories, TV/Addon managers, Subtitles, Player picker, and Download confirmation),
  preventing terminal graphics from bleeding through popup borders and text.
- **Catppuccin Latte light theme & Basic 16-color ANSI light palette**: Tuned Catppuccin Latte
  colors to meet WCAG AA contrast standards on light terminal backgrounds, and introduced a
  specialized 16-color ANSI light fallback palette (`Theme::fallback(true)`) ensuring crisp
  readability without truecolor support.
- **Harmonized modal & picker navigation**: `Home` (jump to first), `End` (jump to last),
  `PageUp` (step -5), and `PageDown` (step +5) support across Browse categories, Theme
  picker, TV Playlist manager, and Addon manager dialogs.
- **Download confirmation dialog key navigation**: `Tab` and `BackTab` (`Shift+Tab`) support
  for toggling between `[ Download ]` and `[ Cancel ]` action buttons in addition to
  `Left`/`Right`.
- **Standardized status durations & helpers**: semantic tick duration constants
  (`STATUS_TICKS_SHORT`, `STATUS_TICKS_DEFAULT`, `STATUS_TICKS_LONG`) and ergonomic
  `AppState` status helper methods replacing magic number literals across all TUI subsystems.
- **Terminal-aware rendering overhaul**:
  - Theme autodetection now actually runs when no explicit theme is configured:
    `NO_COLOR` forces monochrome, truecolor terminals get full RGB palettes,
    strict 256-color terminals get quantized indexed palettes, ANSI-only
    terminals get the fallback palette, and an OSC 11 background query picks
    light/dark variants by measured luminance instead of `COLORFGBG` guesswork.
  - Graphics salvage: terminals that answer kitty/sixel capability probes but
    no cell size (Windows Terminal sixel, iTerm2 over SSH) keep posters via
    default cell metrics instead of falling back to "No Poster".
  - `MOVIEBOX_IMAGE_PROTOCOL=kitty|sixel|iterm2|none` and
    `MOVIEBOX_CELL_SIZE=WxH` overrides, documented in `--help`.
  - Automatic re-probe on focus regain when the initial probe found nothing
    (covers tmux attach after startup), plus a `/probe` slash command.
  - Kitty keyboard protocol (`DISAMBIGUATE_ESCAPE_CODES | REPORT_EVENT_TYPES`)
    for faster unambiguous keys on Ghostty/kitty/foot/WezTerm/Alacritty.
- **Responsive result grid**: results render in two columns at ≥110 columns
  and three at ≥160, with row-based `↑`/`↓`, item-step `←`/`→`, grid-aware
  click mapping, pagination, prefetching, and scroll clamping. Narrow
  terminals keep the classic single-column behavior.
- **Contextual search bar status feedback**: transient status messages (such
  as search clears, provider switching, and cache updates) now display directly
  inside the search bar prompt in accent styling, returning to the mode placeholder
  when the timer expires.
- **Scrollable help overlay**: overflowing help content switches to a scrolled
  single column with position indicator; keyboard and wheel scrolling with
  key-swallowing so shortcuts no longer act behind the overlay.
- **Viewport-adaptive caps**: picker popup rows grow with terminal height
  (4–14), details selector height becomes viewport-proportional, and non-
  graphics terminals get a compact filmstrip poster placeholder.
- **Search editing shortcuts & Tab suggestion completion**: `Tab` auto-completes
  regular search query suggestions in addition to slash commands; `Ctrl+U` clears
  the entire search input line and `Ctrl+W` deletes the previous word.
- **Cursor shape & real input cursor**: `SetCursorStyle::SteadyBar` activates in
  input editing mode on supported terminals (Ghostty, Kitty, WezTerm, foot,
  Alacritty, Windows Terminal) and smoothly restores on normal mode, exit, or panic.
- **Interactive toasts & gauge cancellation**: clicking notification cards dismisses
  them; toast badges display kind icons (`ℹ`/`✔`/`⚠`/`✖`); clicking the active
  download gauge triggers cancellation.
- **Details footer responsive threshold**: single-row footer breakpoint adjusted to
  86 columns in sync with mouse hit-testing, preventing button clipping on 70–85
  column viewports.
- **`f` and `*` favoriting parity**: pressing `f`, `F`, or `*` on Home search results
  or the Favorites landing row now toggles favorite status, matching Details.
- **Streamlined `/toggle-*` slash commands**: consolidated paired `/enable-*` and
  `/disable-*` commands into 4 primary toggles (`/toggle-tv`, `/toggle-addons`,
  `/toggle-bdix`, `/toggle-streaming`) with backward-compatible aliases.

### Changed
- **Framed Zero-Results & Error state cards**: Enclosed empty search results and query error states in styled, centered cards with pill headers (`SearchViewState::Error`, `SearchViewState::NoResults`) and actionable guidance pills (`[Ctrl+P] Switch Provider`, `[r] Retry`, `[Esc] Back`), with dedicated handling for empty `/favorites` queries (`"No favorites saved yet"`).
- **Multi-column result grid gutter arithmetic**: Refined `ResultMetrics` and `result_columns_for` column width calculations to account for explicit 1-character column gutters across 1, 2, 3, and 4-column tiers (with 4-column breakpoint at $\ge 220$ columns).
- **Details footer split threshold synchronization**: Unified the single-row to dual-row footer threshold constant `DETAILS_FOOTER_SPLIT_THRESHOLD` ($86$ cols) across screen rendering (`details.rs`) and mouse click detection (`mouse.rs`).
- **Confirmation dialog button contrast**: Enhanced `[ Download ]` and `[ Cancel ]` button styling in confirmation modals with reverse/bold modifiers on basic terminals for WCAG AA visual contrast.
- **Framed landing favorites deck styling & alignment**: Enclosed the landing Favorites list in a matching bordered card with:
  - Header border offset (`╭─ ★  Favorites ──╮` / `+- *  Favorites --+` on basic) with double-space icon breathing room.
  - 1:1 vertical column alignment between header star `★` and selection pointer `▌` (col x0 + 3), and header text `Favorites` and item titles (col x0 + 6).
  - 1-character right breathing margin for item year/type tags (`2024 Movie `).
- **MovieBox client version spoofing upgrade**: Updated MovieBox client identity spoofing to APK `v4.0.01.0813.03` with version codes `50020117..50020121` (`src/providers/moviebox/crypto.rs`), ensuring full backend compatibility and preventing notice video substitution.
- **Explicit 480p and 360p resolution badges**: Standardized quality tags and badges for `480p` and `360p` across details stream tables and basic terminals, replacing generic `SD` labels.
- **Homepage landing optical spacing & card separation**: Increased vertical breathing margin between ASCII header and search deck, separated search input into a dedicated 3-row framed card, and formatted favorites as an aligned standalone deck with proper vertical rhythm.
- **Streamlined 1-line mode switcher**: Simplified bottom navigation bar to `[Ctrl+S] Stream · [Ctrl+T] TV · [Ctrl+A] Addon      [?] Help [q] Quit`, eliminating duplicate provider text.
- **Clean search suggestion dropdown card**: Removed block background color styling on the outer borders and attached dropdown directly beneath the input prompt, eliminating contrasting 4-sided background halo artifacts.
- **Streamlined search suggestions dropdown**: Removed redundant `[SUGGEST]` badges on regular title queries, replaced tree branch glyphs with clean `▌ ` / `> ` selection indicators, and added full-width active row background styling.
- **Streams list tabular column alignment**: Standardized resolution badges to a uniform 7-column width and aligned file size, codec/media tags, duration, and uploader columns across all resolution tiers.
- **Clean streams section headers**: Replaced solid block badges on stream quality group headers with clean typographic labels (`1080p · 1 option`) eliminating double-box visual clutter.
- **Search card focus state differentiation**: Muted selection highlighting when search bar is in `Editing` mode to maintain clear visual hierarchy.
- **Multi-column search results margin**: Added 1-column right margin on multi-column search results preventing overlap with the vertical scrollbar.
- **Stabilized landing search deck width**: Standardized input deck layout width avoiding horizontal jitter while typing.
- **Clean ghosted placeholder**: Rendered subtle `❯ █ Search movies and series...` prompt when search input is empty in editing mode.
- **Grouped secondary stream media tags**: Secondary stream audio/video codec tags (`DV · ATMOS · HEVC`) now group with subtle `·` text separators rather than heavy boxes, preserving release title space.
- **Responsive Mode Tabs**: Mode tabs on compact viewports dynamically abbreviate to `<76` (`[Ctrl+S] Stream`) and ultra-compact `<58` (`[S] Stream`), protecting them against collisions with the right-aligned utility bar.
- **TV Mode Logo Responsiveness**: Terminal width `<80` triggers the narrower 33-column compact logo to render in TV mode, establishing clean margins on medium setups.
- **Hitbox Splitting**: Synchronized horizontal clicks for mode tabs and the `[?] Help`/`[q] Quit` widgets by enforcing an exact `saturating_sub(19)` boundary layout.
- **Bounded Hitbox**: Addressed dropdown autocompletion leakage. Mouse click events are now strictly clipped inside the computed dynamic bounds of the search suggestion dropdown list.
- **Normal Mode Direct Clear Shortcut (`c` / `C`)**: Pressing `c` or `C` in `InputMode::Normal` clears the active search query instantly and clears search view states when not in an active download.
- **Single-Column Grid Sequencing**: Standardized `<Left>` and `<Right>` movement behaviors in single-column grids so that `jump = 1` enforces granular sequential traversal.
- **Vertical Picker Scaling**: Budget capacity for Picker rows has shifted to `(height - 6).clamp(4, 14)`, effectively granting 14 visible rows on a standard 24-row height layout.
- **Help Layout Rendering Safety**: Wide but vertically short environments safely fall back to scrollable single-column views instead of broken two-column layout.
- **Update Component Layout**: Brought the auto-updater window entirely into parity via `ModalFrame`, granting background area clearing overlays and standard unified outlines.
- **Confirmation Footer Buttons**: Dropped redundant confirmation footers to inject dynamic, formatted `[ Download ] [ Cancel ]` dialog responses seamlessly inside confirmation views.
- **Synopsis Clean Formatting**: Wrapped synopses auto-strip trailing punctuations (`.`, `,`, `!`, `?`, `:`, `;`) before truncating with an ellipsis (`…`).
- **NO_COLOR Adjustments**: Added standard `Modifier::REVERSED` rendering on active item rows across Pickers and Results where `ColorSupport::NoColor` is enforced.
- **CI skip conditions**: `Publish to Crates.io` and `Update Homebrew Formula` now skip gracefully when a Release run completed without publishing a new version; the live-network acceptance test is opt-in via `MOVIEBOX_LIVE_TESTS=1 cargo test --test real_acceptance -- --ignored`.

### Fixed
- **Search suggestions background halo artifact**: Suppressed landing favorites background rendering when search suggestions are active and cleanly aligned dropdown container bounds, eliminating 4-sided dark shadow artifacts around suggestion borders.
- **Search Suggestions Overlay Bleed**: Wrapped search suggestions in an elevated, bordered card with opaque background clearing (`theme.surface0`), and gated terminal graphics poster rendering while typing suggestions, eliminating text and image collisions over background search results.
- **Season Download Stream Error Recovery**: Prevented download queue stalls when resolving individual episode stream URLs fails; failed streams report diagnostics and allow the remaining queue to continue.
- **Favorites & Watch History Corruption Recovery**: Corrupted JSON configuration and history state files now rotate to `.corrupt` backups instead of hard deletion, preserving recoverable user data.
- **MovieBox stream URL replacement with upgrade notice video**: Fixed an issue where the MovieBox backend substituted movie and TV episode stream URLs with a 22-second app upgrade notice video (`1c7de0bd3393702d9191801f15f88f8d.mp4`) when legacy client version headers (`3.0.03.0529.03`) were detected.
- **Mobile User-Agent forwarding to media players & downloader**: Forwarded the spoofed mobile `User-Agent` to external players (`mpv`, `VLC`, `IINA`) and the download HTTP client, preventing HTTP 428/403 errors when streaming or downloading from `bcdn.hakunaymatata.com`.
- **Series multi-resolution discovery fallback**: Enhanced `fetch_collection_resolutions()` to inspect `list` array items when `collectionResolutions` is empty, sorting discovered qualities descending and defaulting to `[1080, 720, 480, 360]`.
- **Details Screen Hit-Testing & Selector Alignment**: Aligned mouse hit-testing regions and season/episode selector column geometry across all compact, medium, and wide terminal tiers.
- **Unicode Display Width in Input Widgets**: Single-line text input fields now compute column offsets using visual Unicode width instead of raw character counts, preventing CJK and multi-byte character overflow from wrapping box borders.
- **M3U Playlist Channel Title Parsing**: Channel titles containing commas in `#EXTINF` metadata are now parsed without truncation, preserving complete channel display names.
- **Subtitle Cache Purge Path Alignment**: Aligned `purge_stale_subtitles` with `resolve_subtitle_dir()` to ensure downloaded subtitle sidecars are cleaned automatically on startup.
- **Android Termux Directory Resolution**: Added comprehensive fallback resolution for config, cache, and data directories when `dirs::*` returns `None` on Android Termux.
- **Android Termux Self-Update Protection**: Added architecture guard preventing Termux from downloading incompatible glibc Linux ARM64 binaries that overwrite working Bionic installations.
- **Windows UNC & Extended Path Escaping**: Preserved leading backslashes on Windows UNC (`\\server\share`) and extended-length (`\\?\`) paths when formatting player arguments.
- **Windows Update Helper `find.exe` Qualification**: Explicitly qualified `%SystemRoot%\System32\find.exe` in the update helper `.bat` script to prevent shadowing by Git for Windows' `find.exe`.
- **Windows NTFS Forbidden Character Sanitization**: Hardened tracker state file generation by stripping control characters and NTFS forbidden characters (`*`, `?`, `"`, `<`, `>`, `|`, `:`).
- **Update Modal Keystroke Fallthrough**: Allowed non-modal navigation keystrokes to fall through to the active screen while the update banner is displayed.
- **Playback & stream resolve state-lock fix (`playback.rs`, `download.rs`)**: Fixed an issue where failed or timed-out 4KHDHub/mirror resolutions left `is_resolving_playback` permanently set to `true`, preventing subsequent playback attempts; added a 15-second resolution timeout and enforced state resets on all error, cancellation, and modal dismissal paths.
- **Poster placeholder overlap on loaded images**: Fixed a rendering bug on the Home results screen where the fallback placeholder widget was unconditionally drawn on top of loaded poster image protocols; the placeholder is now restricted to the `else` branch when an image is still fetching or unavailable.
- **Results view background bleed-through**: Added explicit area clearing for the results chunk before rendering search cards, eliminating ghost text artifacts (such as ASCII headers or landing row labels) bleeding through beneath unselected cards in multi-column grids.
- **Modal key trapping & isolation**: Structural modal gating prevents mode chords (`Ctrl+T`, `Ctrl+A`, `Ctrl+S`, `Ctrl+P`), download cancellation (`x`), and background pane navigation from leaking into open dialogs and inputs.
- **Search dropdown alignment & bleed**: Pinned the suggestion dropdown position to the search bar for stability, fixed layout starvation edge cases allowing background colors to bleed, and padded visual pill badges to standard widths.
- **Search bar view normalization**: Search bar correctly fills terminal width rather than floating when active in empty, No Results, and Error states.
- **Centered text-aligned error cards**: Replaced per-line centering on multiline diagnostic error states with block-level centering containing cleanly left-aligned text, making diagnostics far more readable.
- **Adaptive No Results layout**: Wrapped action pill shortcuts (`[Ctrl+P]`, `[/browse]`, `[Ctrl+U]`) across multiple lines rather than clipping on terminal windows narrower than 76 columns.
- **Text poster fallback clamp**: Fixed an edge case where text-based "No Poster" fallbacks expanded beyond visual limits in compact window modes.
- **Results metadata line truncation**: Added defensive truncation across styled multi-span metadata lines in search results to protect multi-column layouts from overflow crashes.
- **TV & Addon Manager visual polish**: Aligned the selection cursor perfectly in TV playlist manager without introducing a visual shift, and switched the Addon Manager to `render_stateful_widget` enabling deep scrolling with vertical scrollbars.
- **Dual-column Help overlay layout**: Implemented a responsive two-column Help layout for wide viewports (>=90 cols), fitting the entire Help guide on one screen without scrolling.
- **Picker & Stream list mouse offsets**: Rectified a bounds calculation mapping `click_in_picker` to the popup height instead of the active screen area which restricted clicks, and repaired stream-row mouse tracking which was previously ignoring `stream_scroll` offsets.
- **Modal exact clears**: Replaced 3-column halo clear with exact popup bounding box clears, eliminating cutout artifacts on background cards and borders.
- **Favorites landing navigation**: Pressing `Up` from the top favorite item cleanly returns focus to the search bar instead of getting stuck.
- **Manager shortcut harmonization**: `Space` (toggle) and `Delete` (remove) are now supported across both TV Playlist and Addon managers.
- **Details synopsis & theme contrast**: Synopsis text styled with `theme.subtext1` for crisp readability; Nord overlay ramp and TokyoNight border luminance tuned.
- **DNS resolution on Android/Termux and minimal containers**: Replaced the reqwest `hickory-dns` feature flag with a custom resolver (`src/net.rs`) that reads the OS DNS configuration first (`/etc/resolv.conf`, registry on Windows) and falls back to embedded public resolvers (Cloudflare, Google, Quad9) when none exists. The previous approach failed every lookup on platforms without `/etc/resolv.conf` (hickory reads only the hardcoded `/etc/resolv.conf` path); the Termux `$PREFIX/etc/resolv.conf` installer workaround it required is removed.
- **Crash on multibyte titles**: Year stripping sliced remote HTML titles at a raw byte offset and could abort the whole app on CJK/accented characters.
- **Windows self-update never applied**: The staged binary lived inside a temporary directory deleted before the detached `.bat` helper ran, so the update silently never replaced the executable. Staging now persists beside the installed binary, the helper consumes and cleans it, and stale staging artifacts are swept at startup.
- **State file durability**: History/favorites/config writes fsync before rename and never delete the existing destination unless a replacement has succeeded.
- **install.sh correctness**: Install-directory fallback now happens outside the spinner subshell (reported path and shell-rc PATH edits previously used the wrong directory), INT/TERM exit cleanly instead of continuing, the version temp file is cleaned up on failure, PATH/profile matching is literal (regex-safe paths), missing `--version`/`--dir` values fail with clear errors, unknown arguments warn, trailing slashes in `--dir` are normalized, and reinstalling over a running binary avoids `ETXTBSY`.
- **install.ps1 hardening**: TLS 1.2 is enabled independently so older .NET stacks keep it even when TLS 1.3 is unavailable; User PATH updates are written through the registry preserving REG_EXPAND_SZ variables and use exact segment matching; the uninstaller removes its stale User PATH entry; `-Version 0.1.x` is accepted and normalized to `v0.1.x`.
- **Mouse click accuracy**: Search-result hit testing uses the real scroll offset instead of deriving it from the selection, fixing clicks selecting or opening the wrong title after sorting or scrolling partway.
- **Popup geometry parity**: Picker, TV-config, addon-manager, and download-confirm popups share one layout function between renderer and mouse handler, fixing off-by-one-row TV clicks, missed confirmation buttons on short summaries, skewed picker widths, and scroll-blind hit-testing on lists longer than eight rows.
- **Text layout safety**: Details stream rows budget by display width rather than byte length (CJK uploaders/languages no longer overflow); suggestion descriptions, addon names, download gauge status, and playlist URLs truncate to their containers; notification card width no longer panics below 40 columns; addon URL input edits by grapheme clusters so CJK/emoji cursors stay aligned.
- **Poster memory footprint**: Decoded posters are downscaled to ≤512px before caching, decoded/encoded cache sizes were right-sized (roughly 10× less RAM on low-end devices), failed posters retry after 10 minutes instead of being negatively cached forever, and prefetch shares one concurrency limiter instead of creating a new semaphore per scroll batch.
- **Rendering & input polish**: The terminal graphics probe is capped at 400ms and runs off the UI thread (previously up to 2s of blank screen at startup); resize performs a single deferred clear instead of two flashes; grid poster width derives from real terminal cell metrics; the input caret blink keeps animating while typing; the mouse wheel scrolls contextually; `NO_COLOR` now overrides `MOVIEBOX_THEME`; strict 256-color terminals receive quantized indexed palettes rather than RGB sequences.
## [0.1.14] - 2026-08-26

### Added
- **Favorites**:
  - Added a Favorites feature for starring whole movies and series (`src/favorites.rs`, `favorites.json`), independent of watch history and unaffected by `/clear-cache`.
  - Added `*` on the Home screen and `f` / `F` on the Details screen to toggle a title's favorite status, with a `★` indicator on favorited rows and a `[f] Favorite` / `[f] Unfavorite` hint on the Details screen.
  - Added an arrow-navigable Favorites row on the landing screen (Streaming and Addon modes) showing up to 5 recently-starred titles, with a `+N more • /favorites` overflow link; `Down` from the search bar focuses the row, `Enter` opens the selected title, `Esc` releases focus.
  - Added the `/favorites` slash command, mirroring `/history`, to load the full starred list into the results view; `*` unstars the selected row there.
  - Added mouse support for the landing Favorites row (select/open rows, open the full list via the overflow line).
  - Extracted cross-provider title-identity matching into `SubjectIdentity` (`src/models.rs`), now shared by watch history and Favorites so remakes, cross-provider duplicates, and movie/series title collisions are deduplicated identically.

### Fixed
- **External player failure reporting**: Treat every non-zero player exit, including exits after several seconds or without stderr output, as a playback error instead of reconciling false watch progress.
- **Search focus**: Restore Backspace on the Home screen as a reliable way to focus the search input from results or Favorites.
- **Season subtitles**: Remember an explicit subtitle or no-subtitle choice for every episode in a season download.
- **Playback state safety**: Sanitize provider and subject identifiers before using them in tracker state filenames.
- **Cross-platform release validation**: Run all-feature locked builds/tests, binary startup smoke tests, Unix and Windows installer syntax checks, and release-target builds in CI.
- **Release artifact smoke tests**: Execute each native release target's produced binary on its CI runner before archiving.
- **Documentation accuracy**: Document Android intent limitations, the actual Termux binary model, macOS-only IINA support, native runtime verification requirements, and the current automated test count.
- **Windows VLC & player spawn fix**: Removed erroneous `CREATE_NO_WINDOW` flag that suppressed GUI window creation and caused VLC to crash on Windows; normalized Windows backslash subtitle paths in `--sub-file` and gracefully handle subtitle download failures without breaking stream playback.
- **Termux player opener resolution**: Resolve `termux-open`, `termux-open-url`, and `termux-am` directly in `$PREFIX/bin` and static Termux paths; remove broken `/system/bin/am` fallback in unrooted Termux that crashed with Permission Denied (exit code 126).
- **Termux dependency path**: Remove the Android platform-verifier dependency path that caused the v0.1.12 startup panic; real-device confirmation remains required.

## [0.1.13] - 2026-08-21

### Added
- **Production-Grade In-App Self-Update Engine**:
  - Implemented modular self-update architecture (`src/updater/` with `check.rs`, `artifact.rs`, `download.rs`, `verify.rs`, `extract.rs`, and `apply.rs`).
  - Added streaming SHA-256 integrity verification validating exact hash matching against release `SHA256SUMS`.
  - Added hardened archive extraction for `.tar.gz` and `.zip` with strict path traversal protection against `..` components and absolute root paths.
  - Added multi-platform installation strategies: atomic binary replacement with `.old` backup and automatic rollback on Unix/Linux/macOS/Termux, detached helper process on Windows, and Homebrew prefix detection guiding users to `brew upgrade moviebox-tui`.
  - Added active work protection deferring self-update when active video playback or background downloads are running.
  - Connected `[u]` shortcut and `[u] Update Now` button in the existing Update Available modal, preserving visual styling, animations, and dismissal model.
  - Added safe terminal state restoration (`disable_raw_mode`, `LeaveAlternateScreen`, `DisableMouseCapture`, `ShowCursor`) before process exec/restart.
- **Update System Concurrency & Platform Compatibility Architecture**:
  - Added single-flight guard (`is_checking_updates`) ensuring manual (`/update`) and automatic startup checks never spawn duplicate concurrent network requests.
  - Added shared geometry calculation (`UpdateModalLayout`, `update_modal_layout`) guaranteeing 1:1 synchronization between popup rendering and mouse hit testing.
  - Added release asset data modeling (`Release`, `ReleaseAsset`, `TargetPlatform`) with deterministic platform compatibility detection across macOS Universal, Linux x64/arm64, Windows x64/arm64, and Android Termux ARM64.
  - Added dedicated integration test suite `tests/update_lifecycle.rs` testing update single-flighting, error recovery, mouse hit testing, and asset filtering.
- **Comprehensive QA & Regression Test Architecture**:
  - Introduced the initial 132-test automated suite covering critical algorithmic boundaries, end-to-end user journeys, watch history reconciliation & precision progress tracking, cross-mode history audit, in-app self-update lifecycle, real-world release artifact downloads, live SHA-256 verification, genuine version upgrade execution, content & metadata loading pipelines, stale request isolation, active player session lifecycle & duplicate launch protection, dynamic slash command autocomplete (`/download-dir reset`), search/command draft cancellation via `Esc`, error handling, addon manifest validation, mouse interactions, modal dismissals, TUI rendering across terminal size matrices, state reconciliation, crypto HMAC signing, download chunk arithmetic, and URL/stem security.
  - Added structured integration tests in `tests/` (including content, error, TUI, history, Favorites, update, release, cache, player, TV playlist, addon, download, and URL-security suites) and test fixtures (`tests/fixtures/`).
  - Added [`docs/testing.md`](docs/testing.md) detailing test architecture, command references, and manual QA procedures.
- **Playback Tracking & Watch History Progress**:
  - Added real-time playback position tracking for `mpv` with injected tracker script (`moviebox_tracker.lua`) and 5-second periodic state auto-save to disk.
  - Added automatic startup state reconciliation (`reconcile_pending_playback_states`) ensuring watched progress is preserved even when closing the terminal or killing tmux mid-playback.
  - Added two-tone smooth scrub line progress bars (`━─────── 1% (2h 18m left) • Watched 11h ago`) and completion status badges (`[✓ Completed]`, `[✓ Watched]`) in `/history` and Details screens.
  - Added cross-provider title-based history deduplication and auto-resume from the last watched position.
- **Addon Mode Watch History Parity**:
  - Added full watch history support (`/history`) in Addon Mode matching Streaming Mode, enabling seamless watch progress tracking, scrub bars, and completion badges for community HTTP addon content.
- **Pluggable Provider Trait & Capability Architecture**:
  - Formalized the public `Provider` and `ReleaseProvider` traits across all built-in scrapers (`MovieBox`, `4KHDHub`, `CircleFTP`, `DhakaFlix`, and `Addons`).
  - Added `ProviderCapabilities` (`supports_search`, `supports_pagination`, `supports_series`, `supports_subtitles`, `supports_homepage`) and `MovieBoxService::capabilities()` for dynamic capability reporting.
  - Added structured `ProviderError` boundaries (`Network`, `RateLimited`, `NotFound`, `Parsing`, `Unavailable`) with `.user_message()` for consistent error notifications.
- **Theme System Expansion & Official Color Calibration**:
  - Added official **Dracula**, **Gruvbox**, and **Rosé Pine** themes to the `/theme` picker alongside Catppuccin and Nord.
  - Added alias parsing support for `"dracula"`, `"gruvbox"`, `"rose-pine"`, and `"catppuccin"`.
  - Guaranteed 100% transparent terminal compatibility across all themes with zero background opacity overrides.
  - Fixed modal backdrop rendering by removing fullscreen screen clearing when opening `/theme`.
  - Optimized live preview navigation to eliminate unnecessary disk I/O on arrow key navigation.
- **Universal Multi-OS Player Detection & Flathub/Snap Compatibility**:
  - Added sub-millisecond, filesystem-backed player probing across Linux (Flathub, Flatpak exports, Snap, and Native), macOS (Homebrew, MacPorts, App Bundles), Windows (Program Files, WinApps, Scoop, Chocolatey, WinGet), and Android (Termux).
  - Fixed Flathub/Flatpak VLC detection failure by adding direct probes for `~/.local/share/flatpak/exports/bin/org.videolan.VLC` and `/var/lib/flatpak/exports/bin/org.videolan.VLC`.
  - Added full Flathub/Flatpak and Snap compatibility for MPV (`io.mpv.Mpv`, `/snap/bin/mpv`).
  - Centralized player process construction (`build_player_process_command`) and standardized subtitle flag arguments (`--sub-file=<path>`) across all platforms.
- **Codebase Optimization & Comprehensive Caching Architecture**:
  - Centralized application paths (`config_dir`, `data_dir`, `cache_dir`, `logs_dir`, `scripts_dir`, `playback_state_dir`) in `src/config.rs`.
  - Added dedicated disk caching for Addon Mode stream aggregation (`2h` TTL), catalog `/browse` presets (`1h` TTL), and verified manifests (`24h` TTL).
  - Added search pagination caching (`search_{hash}_{page}.json`) preventing redundant API calls when navigating multi-page search results.
  - Eliminated redundant `reqwest::Client` allocations in background poster pipelines in favor of the shared `service.http_client()`.
  - Streamlined `MovieBoxService` usage across background tasks and removed redundant `addon_client` field from `AppState`.
  - Centralized formatting utilities (`format_file_size`, `format_duration`) in `src/tui/text.rs`.
  - Modernized `Config` loading and persistence with safe, standard Serde derives.
- **Addon Mode (Community HTTP Addons)**:
  - Added full support for community HTTP addon manifests (`/manifest.json`, `/catalog`, `/meta`, `/stream`) with dedicated `Ctrl+A` mode switching.
  - Pre-installed and locked Cinemeta out-of-the-box as the default core metadata provider with zero API keys required.
  - Added interactive Addon Manager dialog (`/addons`, `Ctrl+P` in Addon Mode) with one-click enabling, removal, and manifest URL adding.
  - Added concurrent multi-addon stream resolution aggregating playable releases from all enabled stream addons.
  - Added a smart runtime torrent detector that automatically detects if an addon's streams are 100% blocked raw torrents (e.g., Torrentio without Debrid) and flashes a UI warning toast that only HTTP streams are supported.
- **Addon Mode `/browse` & Curated Catalog Exploration**:
  - Added `/browse` support in Addon Mode with a minimal, organized 4-preset catalog picker (`Top Movies`, `Top Series`, `Top Rated Movies`, `Top Rated Series`).
  - Added direct catalog fetching (`/catalog/{type}/{id}.json`) with poster hydration, details navigation, stream resolution, and `/reload` support.
- **Strict Slash Command Guarding & Guidance**:
  - Intercepted all `/` slash commands to guarantee zero remote catalog network requests.
  - Added warning toast notifications for unrecognized slash commands (`"Command '/xyz' is not recognized. Type '/' to view available commands."`).
  - Added platform-aware mode-guidance toasts (`^T` / `^S` / `^A` on macOS, `Ctrl+T` / `Ctrl+S` / `Ctrl+A` on Linux/Windows) for mode-restricted commands.
- **Active Mode & Provider State Persistence**:
  - Added `active_mode` configuration field in `config.json` automatically persisting and restoring the last active mode (`streaming`, `tv`, `addon`) and active provider across app restarts.
- **Configurable Mode Navigation**:
  - Added `/enable-streaming`, `/disable-streaming`, `/enable-tv`, and `/disable-tv` slash commands alongside `/enable-addons` and `/disable-addons`.
  - Enforced safety validation ensuring at least one mode remains active and gracefully migrating focus when disabling the current mode.
- **Dynamic Multi-Source Host & Resolver Resolution**:
  - Added 100% dynamic domain-based host extractor (`extract_domain_label`) and stream tag parser (`detect_stream_host`) identifying and formatting direct hosts (Pixeldrain, Hubcloud, Fast Download, Google Drive, Mega, etc.) and debrid resolvers without hardcoded tables.
- **Full Emoji & Symbol Sanitization**:
  - Added `strip_emojis` and `clean_stream_text` sanitizing all raw stream titles, release names, source labels, and languages from community addons for clean terminal alignment without broken characters.
  - Standardized checkbox representations to clean ASCII `[x] / [ ]`.
- **Complete Mouse Navigation**:
  - Added dynamic footer hitboxes for `[Ctrl+S] Streaming`, `[Ctrl+T] TV`, `[Ctrl+A] Addons`, `[Ctrl+P] {Provider}`, `[?] Help`, `[q] Quit`.
  - Added complete mouse click support for Addon Manager modal and browse popups.

### Fixed
- **Windows MSVC Static CRT Linking (`+crt-static`)**:
  - Configured `target-feature=+crt-static` in `.cargo/config.toml` for `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`, statically embedding the C runtime to eliminate external `VCRUNTIME140.dll` dependency and resolve `0xC0000135` (`STATUS_DLL_NOT_FOUND`) on clean Windows installations.
- **Cross-Platform Installer Polish & Windows In-Memory Execution**:
  - Replaced file-based execution commands in Windows documentation with the in-memory stream pipeline (`irm ... | iex`) to eliminate `PSSecurityException` execution policy blocks.
  - Added immediate active process `$env:PATH` update in `install.ps1` so the command is recognized in the current shell session without terminal restart.
  - Replaced rigid fixed-width boxed summary tables with responsive, borderless hero layouts across both `install.ps1` and `install.sh`, preventing broken box-drawing characters and layout overflow on narrow screens.
- **Pending History Reconciliation Order**:
  - Sorted pending Lua tracker state files chronologically during startup reconciliation to guarantee correct playback state replay order.
- **MovieBox Title Sanitization (DEF-02)**:
  - Fixed destructive title truncation where leading bracket tags (`[Dub]`, `[1080p]`, `[RAW]`) and titles starting with parentheses (e.g. `(500) Days of Summer`) were stripped down to empty strings.
  - Preserved release years in parentheses (`Inception (2010)`) and added a fallback safeguard returning the trimmed original title if sanitization ever results in an empty string.
- **Watch History Identity & Deduplication Collisions (DEF-03)**:
  - Enforced `stype` separation in `HistoryManager::is_same_show` so Movies and TV Series sharing identical titles (e.g. `Home`) never overwrite one another.
  - Enforced strict canonical identity (`provider + subject_id`), preventing cross-provider conflicts and ensuring remakes with differing release years remain distinct entries.
- **Background Episode Playback State Reconciliation (MISS-01)**:
  - Fixed a state loss bug in `reconcile_pending_playback_states` where a completed episode's watched status was discarded if the user had already advanced to a subsequent episode before the state file was processed.
- **Windows MPV Script Options Path Escaping (DEF-04)**:
  - Fixed path corruption in MPV's `--script-opts` on Windows by normalizing backslashes (`\`) to forward slashes (`/`), preventing MPV escape sequence parsing from corrupting `state_file` paths in `moviebox_tracker.lua`.
- **M3U Single-Quoted Attribute Support (DEF-07)**:
  - Extended `M3UParser` attribute extraction to support both single-quoted (`tvg-id='...'`) and double-quoted attributes, preserving channel IDs, logos, and groups across varied IPTV playlists.
- **Continuous OS-Level SIGINT Handling (DEF-05)**:
  - Wrapped `tokio::signal::ctrl_c()` in a continuous background loop to ensure repeated non-interactive OS signals are reliably handled.
- **Subtitle Prefetch Fallback (DEF-08)**:
  - Reduced subtitle download timeout from 30s to 8s to prevent unnecessary startup delays when launching external players if a subtitle mirror hangs.
- **Addon Stream Sorting & Rendering**:
  - Fixed addon streams randomly scrambling on UI hover when sizes are tied by adding a secondary stable sort based on the mirror label.
  - Fixed misleading `0MB` stream sizes for community addons that omit video sizes by cleanly rendering `--` instead.
- **Terminal Race Condition & Blank Screen on `/clear-cache`**:
  - Replaced physical terminal clear with a soft image refresh when executing `/clear-cache`, resolving a race condition with terminal emulators that swallowed the full Home screen render and caused the screen to go completely blank after a few seconds.
  - Added comprehensive state isolation preventing search queries, results, and details states from lingering after cache clears.
  - Sanitized slash command input handling to prevent visual query glitches.
  - Replaced standard status messages with elevated toast notifications for cache actions.
- **Atomic Mode Highlight & Single Active Selection**:
  - Added canonical `AppMode` enum (`Streaming`, `Tv`, `Addon`) and atomic state transitions guaranteeing that only one active mode is highlighted in the bottom dock at any time.
  - Hardened state isolation with automatic cleanup across mode switches.
- **Notification Readability & Word-Boundary Wrapping**:
  - Replaced horizontal middle-truncation with unicode display-width aware word wrapping (`wrap_text`).
  - Added adaptive width (up to 72 chars) and dynamic height scaling with guaranteed unbroken rounded borders.
- **Resilient Addon Metadata & Fallbacks**:
  - Added flexible visitors and serde aliases for `genres`, `cast`, `director`, `imdbRating`, `releaseInfo`, and `runtime` preventing deserialization failures across varied community addon JSON schemas.
  - Added multi-tier fallback resolution in the Details screen to guarantee titles, release years, synopsis, and posters are always preserved from search results and previews.
- **Android / Termux TLS Certificate Compatibility**:
  - Switched `reqwest` to use pure-Rust embedded `webpki-roots` certificate verification, resolving `rustls-platform-verifier` crashes and panics in non-JVM Android CLI environments like Termux.
- **Transparent Stream & Search Diagnostics**:
  - Replaced misleading generic `"No matches"` and `"Rate Limit"` errors with truthful, contextual diagnostics: `"No stream sources available on {provider}"`, `"Network connection failed to {provider}"`, `"Rate limited by {provider}"`, and `"Episode S{season}E{episode} is not listed on {provider}"`.
  - Added helpful actionable hints (`Press Ctrl+P to try another provider, or r to refresh`).
- **Rate-Limiting & Concurrency Hardening**:
  - Added HTTP 429 `Retry-After` header parsing with bounded exponential backoff in `MovieBoxClient`.
  - Added semaphore concurrency limiting (`Semaphore::new(2)`) during parallel episode page resolution to prevent burst requests from tripping provider rate limiters.
- **Addon Mode Series Hierarchy & Episode Stream Isolation**:
  - Fixed series misclassification as movies in Addon Mode when metadata omitted the `videos` array by ensuring canonical season structures and series-first metadata endpoint prioritization.
  - Added regex and token-based episode stream isolation (`parse_season_episode`) in `stream_item_to_release`, preventing cross-episode stream pollution (e.g. S01E06 streams appearing when viewing S01E08).
  - Added preservation of `episodeNumbers` arrays from addon metadata in the season list state.
- **Direct Addon & BDIX Playback & Download Dispatch**:
  - Fixed Addon and BDIX playback and download routing in `handle_playback` and `handle_download` to dispatch directly to external media players and the chunk downloader, preserving custom HTTP headers (`behaviorHints.headers`) and source labels without unnecessary Moviebox API subtitle timeouts.
- **Selector Tab Preservation in Standard Displays**:
  - Maintained visibility of Audio Languages, Seasons, and Episodes selector tabs side-by-side in standard ~80-column terminals when focusing Streams, preventing tabs from disappearing when 0 streams are available.

### Changed
- **Modular TV Provider Architecture**:
  - Reorganized Live TV / IPTV provider into a dedicated module directory (`src/providers/tv/`) with separated `models.rs` and `parser.rs`.
- **Core Infrastructure Consolidation**:
  - Centralized atomic file operations (`atomic_write_file`, `atomic_write_file_async`), MD5 digest formatting (`md5_hex`), and text extraction helpers in `cache.rs` and `service.rs`.
  - Centralized application paths, border type resolution, and mode status announcements across TUI modules.
- **Addon Manager UI Optimization**:
  - Implemented full cursor navigation (`Left`/`Right` keys) and inline editing (`Backspace`/`Delete`) for the Addon Manager input field.
  - Implemented a scrolling viewport renderer for the Addon Manager input, allowing editing of very long manifest URLs without wrapping or truncation.
  - Compacted the Addon Manager dialog with an aligned two-tier layout placing `[ Add Manifest URL ]` and `[ Done ]` action buttons side-by-side.
- **Multi-System Core Module Decoupling**:
  - Promoted `player.rs` (process management & detection), `config.rs` (shared configuration), and `updater.rs` (release checks) to core modules in `src/`, preparing the architecture for upcoming CLI and GUI frontends with full backward compatibility.

### Documentation
- **Streamlined README & Controls Guide**:
  - Transformed `README.md` into a focused landing page with measured `~5 MB RAM` benchmark data, defensible value propositions, and direct links to deep guides in `docs/`.
  - Created standalone `docs/controls.md` covering all keyboard shortcuts, mouse controls, and slash commands.
  - Added a 3-phase project roadmap: Terminal UI (TUI) -> Command-Line Interface (CLI) -> Desktop GUI Client.
  - Added a community-first feedback and support section with optional crypto donation options.

## [0.1.12] - 2026-08-15

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
  - Structured Series downloads under `<base_dir>/Series/<Title>/Season <N>/<Title> - S<N:02>E<E:02>.<ext>` and Movies under `<base_dir>/Movies/<Title>/<Title>.<ext>`.
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
- **Native Graphics & Single Standardized 'No Poster' Placeholder**:
  - Replaced redundant dual labels (`Poster unavailable` / `No Art`) and noisy halfblock mosaic fallback with a single clean, centered `No Poster` label across search results, details, and history on non-graphics terminals.
  - Eliminated ANSI block characters, yellow/white selection redraw bars, and unnecessary background image downloads on basic terminals.
  - Preserved full native high-resolution graphical rendering on Sixel, Kitty, and iTerm2 supported terminals.
  - Added `MOVIEBOX_NO_IMAGE=1` environment override to disable image probing on slow or headless sessions.
- **Next-Gen Multi-Tiered Animated Installers (`install.sh` & `install.ps1`)**:
  - Multi-tier progressive rendering with official MovieBox branding and Catppuccin Mocha aesthetic.
  - Live smooth Braille spinners (`⠋ ⠙ ⠹ ...`), SHA256 cryptographic verification against `SHA256SUMS`, and media player ecosystem detection.
  - 100% sudo-less user-level installation into `~/.local/bin` (or `%LOCALAPPDATA%\Programs\MovieBox-Tui\bin` on Windows) with automatic non-destructive shell PATH integration and zero password prompts.
  - Added full CLI flags: `--version <tag>`, `--dir <path>`, `--force`, `--dry-run`, and `--uninstall`.
- **Explicit Download Directory Autocomplete Hints**:
  - Added `/download-dir <path>` slash command suggestion with clear action descriptions (`Set custom folder (e.g. ~/Movies)` vs. `View current download folder`).
  - Added friendly guidance notification if a user inputs literal `<path>` placeholders.

### Fixed
- **Custom Download Directory Container Hierarchy**:
  - Ensured custom download directories always maintain the standardized `MovieBox-TUI` root container (`MovieBox-TUI/Movies/...` and `MovieBox-TUI/Series/...`) without duplicating if already named `MovieBox-TUI`.
- **Multiline Notification Toast Rendering**:
  - Upgraded notification toast layout to compute dynamic height and wrap multiline messages per line cleanly without horizontal middle-truncation across newlines.
  - Sanitized notification folder paths by substituting home directory with `~`.
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
  - Fixed "No Poster" placeholders in `/history` by automatically resolving missing cover URLs and decoding posters in the background.
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
- **State Decomposition**: Split the monolithic application state into specialized domain state structs.
- **Strict Verification Gates**: Enforced workspace lint checks, static analysis, and testing.
