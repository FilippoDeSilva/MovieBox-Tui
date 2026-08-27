# Controls & Shortcuts

MovieBox-TUI is designed for fast keyboard navigation with complete mouse support throughout the interface. You can press `?` anywhere inside the application to open the mode-aware interactive help dialog.

## Global Shortcuts

| Key | Action |
| :--- | :--- |
| **`↑` / `↓` / `k` / `j`** | Navigate lists, search results, or move cursor up/down |
| **`←` / `→` / `h` / `l`** | Move text input cursor, step wide grid columns, or switch Details panes (Audio/Seasons/Episodes/Streams) |
| **`Home` / `End` / `g` / `G`** | Jump to start / end of list or search results (auto-fetches next page), or move cursor to beginning / end of input line |
| **`PageUp` / `PageDown`** | Scroll search results, lists, and modal pickers by visible page height (or scroll help overlay) |
| **`Enter`** | Open, play, or confirm the selected item |
| **`Space` / `P`** | Direct resume playback for recorded season/episode on `/history` items |
| **`Esc`** | Focus search input (when results present), dismiss popup dialog, or return to landing |
| **`Tab` / `Shift+Tab`** | Auto-complete suggestion / command; switch details panes; toggle dialog buttons |
| **`Backspace`** | Delete character before cursor, or return focus to search bar from results |
| **`Delete`** | Delete character at cursor in text inputs, or remove entry in TV/Addon managers |
| **`Ctrl+U`** | Clear entire input line (Search, TV URL, Addon URL) |
| **`Ctrl+W`** | Delete backward word in text inputs |
| **`Ctrl+S`** | Switch to standard **Streaming Mode** |
| **`Ctrl+T`** | Toggle / switch to **TV Mode** |
| **`Ctrl+A`** | Toggle / switch to **Addon Mode** |
| **`?`** | Open interactive in-app help menu |
| **`Ctrl+C` / `q`** | Quit application and restore terminal |

## Text Input & Cursor Editing

Text editing across Search, TV Playlist Manager, and Addon Manager uses a unified grapheme-safe input engine:

| Key | Action |
| :--- | :--- |
| **`Left` / `Right`** | Move text cursor one grapheme cluster left or right |
| **`Home` / `End`** | Jump cursor directly to the beginning or end of the input line |
| **`Backspace`** | Delete the grapheme cluster immediately before the cursor |
| **`Delete`** | Delete the grapheme cluster at the cursor position |
| **`Ctrl+W`** | Delete the preceding word (up to space or punctuation delimiter) |
| **`Ctrl+U`** | Clear the entire input buffer |
| **`Tab`** | Auto-complete active search suggestion or slash command |
| **`Enter`** | Submit search query, save TV playlist URL/path, or verify and install Addon manifest |
| **`Esc`** | Cancel input, dismiss input prompt, or clear search buffer |

## Modal Dialogs & Pickers

All popup dialogs (Theme picker, Browse categories, Player picker, TV Manager, Addon Manager, Download Confirmation) support standard keyboard controls:

- **`↑` / `↓`**: Move selection up / down by one item.
- **`Home` / `End`**: Jump immediately to the first or last item in the list.
- **`PageUp` / `PageDown`**: Step up or down by 5 items.
- **`Enter`**: Confirm selection, activate entry, or submit dialog.
- **`Esc`**: Dismiss popup dialog without applying changes.
- **Download Confirmation Dialog**:
  - **`Tab` / `Shift+Tab` / `BackTab`**: Toggle active selection between `[ Download ]` and `[ Cancel ]`.
  - **`Left` / `Right`**: Switch between `[ Download ]` and `[ Cancel ]`.
  - **`Enter`**: Confirm the currently focused action.
  - **`Esc`**: Cancel and close the confirmation dialog.
## Mode-Specific Controls

### Streaming Mode
- **`Ctrl+P`**: Cycle content providers (`MovieBox` $\to$ `4KHDHub` $\to$ `BDIX`).
- **`←` / `→` / `h` / `l` / `Tab` / `Shift+Tab`**: Switch Details screen selector panes (Audio Languages, Seasons, Episodes, Streams).
- **`Enter`**: Play with default player.
- **`o`**: Open player selection picker for the current stream.
- **`d`**: Download current episode or full season batch.
- **`r`**: Refresh search results / stream list.
- **`*` / `f`**: Favorite / unfavorite the selected title on the Home screen.
- **`f`**: Favorite / unfavorite the open title on the Details screen.
- **`/browse`**: Open curated browse categories (Trending, Popular, Top Rated, etc.).
- **`/history`**: Open watch history (`Space` or `P` to instantly resume recorded episode/movie).
- **`/favorites`**: Open your starred titles.
- **`/clear`**: Clear active search query and return to landing.
### TV Mode (Live IPTV)
- **`Enter`**: Play selected TV channel immediately.
- **`o`**: Open player selection picker for the channel.
- **`r`**: Reload all active M3U playlist sources.
- **`/config`**: Open TV Playlist Manager dialog (`Space` to activate, `Delete`/`d` to remove).
- **`/list`**: Show all loaded channels.

### Addon Mode (HTTP Addons)
- **`Ctrl+P` / `/config`**: Open Addon Manager dialog (`Space` to toggle, `Delete`/`d` to remove).
- **`←` / `→` / `h` / `l` / `Tab` / `Shift+Tab`**: Switch Details screen selector panes (Seasons, Episodes, Streams).
- **`Enter`**: Select title or play resolved stream.
- **`o`**: Open player selection picker for the stream.
- **`d`**: Download HTTP stream release.
- **`r`**: Refresh addon catalog search results.
- **`*` / `f`**: Favorite / unfavorite the selected title on the Home screen.
- **`f`**: Favorite / unfavorite the open title on the Details screen.
- **`/history`**: Open watch history (`Space` or `P` to instantly resume recorded episode/movie).
- **`/favorites`**: Open your starred titles.
- **`/clear`**: Clear active search query and return to landing.
## Mouse Controls

| Action | Result |
| :--- | :--- |
| **Click search bar** | Enter search input mode |
| **Click suggestion item** | Search for that suggestion immediately |
| **Click search result row** | Select item and load preview; click again to open full details |
| **Click Favorites row (landing)** | Select a starred title; click again to open it |
| **Click "+N more • /favorites"** | Open the full favorites list |
| **Click audio / season / episode / stream** | Switch audio language, change season, select episode, or start playback |
| **Click footer buttons** | Switch provider / mode, open help (`[?]`), or quit (`[q]`) |
| **Click modal buttons** | Choose a theme, subtitles, player, or confirm actions |
| **Click outside a modal** | Dismiss popup dialog |

## Slash Commands

Type these commands directly into the search bar:

| Command | Applicable Mode | Action |
| :--- | :--- | :--- |
| `/browse` | Streaming / Addon | Browse curated views (Trending, Popular) or Addon catalogs (Top Movies, Top Series) |
| `/history` | Streaming / Addon | View watch history with latest progress |
| `/favorites` | Streaming / Addon | View all starred titles |
| `/list` | TV | View live TV channels |
| `/config` | TV / Addon | Manage IPTV playlists (TV Mode) or configure HTTP addons (Addon Mode) |
| `/download-dir` | All | View, change, or reset the download directory |
| `/theme` | All | Open theme picker (Mocha, Latte, Macchiato, Frappe, Nord, TokyoNight, Dracula, Gruvbox, RosePine) |
| `/clear-cache` | All | Clear temporary cache files |
| `/update` | All | Check if a new release is available on GitHub |
| `/toggle-update` | All | Toggle automatic startup update checks |
| `/toggle-streaming` | All | Toggle Streaming Mode navigation in bottom dock (aliases: `/enable-streaming`, `/disable-streaming`) |
| `/toggle-tv` | All | Toggle TV Mode navigation in bottom dock (aliases: `/enable-tv`, `/disable-tv`) |
| `/toggle-addons` | All | Toggle Addon Mode navigation in bottom dock (aliases: `/enable-addons`, `/disable-addons`) |
| `/toggle-bdix` | Streaming | Toggle BDIX FTP sources (aliases: `/enable-bdix`, `/disable-bdix`) |
| `/probe` | All | Re-run terminal graphics detection and report the result |
| `/github` | All | Open the project repository |

## Help Overlay

- Open with `?`; close with `?`, `Esc`, or `q`.
- `↑`/`↓`, `PageUp`/`PageDown`, and the mouse wheel scroll long content.
- Other keys are ignored while help is open.

## Wide-Terminal Grid

On terminals at least 110 columns wide, search results render in two
columns (three at 160+). `↑`/`↓` move one visual row, `←`/`→` move one
item, and clicks map through column bounds. Narrower terminals keep the
classic single-column list where `←`/`→` jump a full page.
