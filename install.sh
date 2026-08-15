#!/usr/bin/env bash
set -euo pipefail

APP_NAME="MovieBox-Tui"
BIN_NAME="moviebox-tui"
REPO="mesamirh/MovieBox-Tui"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

VERSION=""
CUSTOM_DIR=""
FORCE=0
DRY_RUN=0
NO_MODIFY_PATH=0
UNINSTALL=0

for arg in "$@"; do
    case "$arg" in
        --version=*|-v=*)
            VERSION="${arg#*=}"
            ;;
        --version|-v)
            shift
            VERSION="${1:-}"
            ;;
        --dir=*)
            CUSTOM_DIR="${arg#*=}"
            ;;
        --dir)
            shift
            CUSTOM_DIR="${1:-}"
            ;;
        --force|-f)
            FORCE=1
            ;;
        --dry-run)
            DRY_RUN=1
            ;;
        --no-modify-path)
            NO_MODIFY_PATH=1
            ;;
        --uninstall)
            UNINSTALL=1
            ;;
        --help|-h)
            cat << 'EOF'
MovieBox-TUI Installer

USAGE:
    curl -fsSL https://raw.githubusercontent.com/mesamirh/MovieBox-Tui/main/install.sh | bash [OPTIONS]
    ./install.sh [OPTIONS]

OPTIONS:
    -v, --version <tag>    Install a specific version (e.g. v0.1.11)
        --dir <path>       Install binary to a custom directory
    -f, --force            Reinstall even if already at the latest version
        --dry-run          Perform preflight checks without writing files
        --no-modify-path   Do not modify shell profile configuration
        --uninstall        Uninstall MovieBox-TUI from your system
    -h, --help             Show this help message
EOF
            exit 0
            ;;
    esac
done

IS_TTY=0
if [ -t 1 ] && [ -t 0 ]; then
    IS_TTY=1
fi

IS_COLOR=0
if [ -z "${NO_COLOR:-}" ] && [ "${TERM:-}" != "dumb" ] && [ -t 1 ]; then
    IS_COLOR=1
fi

if [ "$IS_COLOR" -eq 1 ]; then
    C_RESET="\033[0m"
    C_BOLD="\033[1m"
    C_DIM="\033[2m"
    C_CYAN="\033[36m"
    C_BOLD_CYAN="\033[1;36m"
    C_GREEN="\033[32m"
    C_BOLD_GREEN="\033[1;32m"
    C_YELLOW="\033[33m"
    C_BOLD_YELLOW="\033[1;33m"
    C_RED="\033[31m"
    C_BOLD_RED="\033[1;31m"
    C_MAGENTA="\033[35m"
    C_BLUE="\033[34m"
    CURSOR_HIDE="\033[?25l"
    CURSOR_SHOW="\033[?25h"
else
    C_RESET=""
    C_BOLD=""
    C_DIM=""
    C_CYAN=""
    C_BOLD_CYAN=""
    C_GREEN=""
    C_BOLD_GREEN=""
    C_YELLOW=""
    C_BOLD_YELLOW=""
    C_RED=""
    C_BOLD_RED=""
    C_MAGENTA=""
    C_BLUE=""
    CURSOR_HIDE=""
    CURSOR_SHOW=""
fi

cleanup() {
    printf "%b" "$CURSOR_SHOW" 2>/dev/null || true
    if [ -n "${TMP_DIR:-}" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

log_step() {
    printf "  %b%s%b %s\n" "$C_BOLD_CYAN" "→" "$C_RESET" "$1"
}

log_success() {
    printf "  %b%s%b %s\n" "$C_BOLD_GREEN" "✔" "$C_RESET" "$1"
}

log_warn() {
    printf "  %b%s%b %s\n" "$C_BOLD_YELLOW" "⚠" "$C_RESET" "$1"
}

log_error() {
    printf "  %b%s%b %s\n" "$C_BOLD_RED" "✖" "$C_RESET" "$1" >&2
}

print_header() {
    if [ "$IS_COLOR" -eq 1 ] && [ "$IS_TTY" -eq 1 ]; then
        printf "%b\n" "$CURSOR_HIDE"
        printf "%b" "$C_BOLD_CYAN"
        local lines=(
            "███╗   ███╗  ██████╗  ██╗   ██╗ ██╗ ███████╗ ██████╗   ██████╗  ██╗  ██╗"
            "████╗ ████║ ██╔═══██╗ ██║   ██║ ██║ ██╔════╝ ██╔══██╗ ██╔═══██╗ ╚██╗██╔╝"
            "██╔████╔██║ ██║   ██║ ██║   ██║ ██║ █████╗   ██████╔╝ ██║   ██║  ╚███╔╝ "
            "██║╚██╔╝██║ ██║   ██║ ╚██╗ ██╔╝ ██║ ██╔══╝   ██╔══██╗ ██║   ██║  ██╔██╗ "
            "██║ ╚═╝ ██║ ╚██████╔╝  ╚████╔╝  ██║ ███████╗ ██████╔╝ ╚██████╔╝ ██╔╝ ██╗"
            "╚═╝     ╚═╝  ╚═════╝    ╚═══╝   ╚═╝ ╚══════╝ ╚═════╝   ╚═════╝  ╚═╝  ╚═╝"
        )
        for line in "${lines[@]}"; do
            printf "%s\n" "$line"
            sleep 0.02
        done
        printf "%b%b%s%b\n\n" "$C_DIM" "                     MovieBox-Tui Installer" "$C_RESET" "$CURSOR_SHOW"
    else
        cat << 'EOF'
███╗   ███╗  ██████╗  ██╗   ██╗ ██╗ ███████╗ ██████╗   ██████╗  ██╗  ██╗
████╗ ████║ ██╔═══██╗ ██║   ██║ ██║ ██╔════╝ ██╔══██╗ ██╔═══██╗ ╚██╗██╔╝
██╔████╔██║ ██║   ██║ ██║   ██║ ██║ █████╗   ██████╔╝ ██║   ██║  ╚███╔╝ 
██║╚██╔╝██║ ██║   ██║ ╚██╗ ██╔╝ ██║ ██╔══╝   ██╔══██╗ ██║   ██║  ██╔██╗ 
██║ ╚═╝ ██║ ╚██████╔╝  ╚████╔╝  ██║ ███████╗ ██████╔╝ ╚██████╔╝ ██╔╝ ██╗
╚═╝     ╚═╝  ╚═════╝    ╚═══╝   ╚═╝ ╚══════╝ ╚═════╝   ╚═════╝  ╚═╝  ╚═╝
                     MovieBox-Tui Installer

EOF
    fi
}

run_spinner() {
    local message="$1"
    shift
    local cmd=("$@")

    if [ "$IS_TTY" -ne 1 ] || [ "$IS_COLOR" -ne 1 ]; then
        log_step "$message..."
        if ! "${cmd[@]}"; then
            log_error "$message failed."
            return 1
        fi
        return 0
    fi

    local spin_chars=("⠋" "⠙" "⠹" "⠸" "⠼" "⠴" "⠦" "⠧" "⠇" "⠏")
    local spin_count=${#spin_chars[@]}
    local spin_idx=0

    local tmp_out
    tmp_out=$(mktemp)

    printf "%b" "$CURSOR_HIDE"
    "${cmd[@]}" > "$tmp_out" 2>&1 &
    local pid=$!

    while kill -0 "$pid" 2>/dev/null; do
        local frame="${spin_chars[$spin_idx]}"
        printf "\r\033[K  %b%s%b %s..." "$C_CYAN" "$frame" "$C_RESET" "$message"
        spin_idx=$(( (spin_idx + 1) % spin_count ))
        sleep 0.08
    done

    wait "$pid"
    local exit_code=$?
    printf "%b" "$CURSOR_SHOW"

    if [ "$exit_code" -eq 0 ]; then
        printf "\r\033[K"
        rm -f "$tmp_out"
        return 0
    else
        printf "\r\033[K"
        log_error "$message failed:"
        cat "$tmp_out" >&2
        rm -f "$tmp_out"
        return "$exit_code"
    fi
}

do_uninstall() {
    print_header
    log_step "Uninstalling $APP_NAME..."
    
    local found=0
    local target_paths=(
        "$HOME/.local/bin/$BIN_NAME"
        "/usr/local/bin/$BIN_NAME"
        "${PREFIX:-}/bin/$BIN_NAME"
    )

    if command -v "$BIN_NAME" >/dev/null 2>&1; then
        local current_path
        current_path=$(command -v "$BIN_NAME")
        target_paths+=("$current_path")
    fi

    for path in "${target_paths[@]}"; do
        if [ -n "$path" ] && [ -f "$path" ]; then
            if [ -w "$path" ] || [ -w "$(dirname "$path")" ]; then
                rm -f "$path"
                log_success "Removed $path"
                found=1
            elif command -v sudo >/dev/null 2>&1; then
                sudo rm -f "$path"
                log_success "Removed $path (with sudo)"
                found=1
            fi
        fi
    done

    if [ "$found" -eq 1 ]; then
        log_success "$APP_NAME was successfully uninstalled."
    else
        log_warn "No installed binary of $BIN_NAME was found."
    fi
    exit 0
}

if [ "$UNINSTALL" -eq 1 ]; then
    do_uninstall
fi

print_header

command -v curl >/dev/null 2>&1 || { log_error "curl is required but not installed. Please install curl."; exit 1; }
command -v tar >/dev/null 2>&1 || { log_error "tar is required but not installed. Please install tar."; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"
IS_TERMUX=0

if [ -n "${PREFIX:-}" ] && [[ "$PREFIX" == *com.termux* ]]; then
    IS_TERMUX=1
fi

if [ "$OS" = "Darwin" ]; then
    FILE="MovieBox_macOS_Universal.tar.gz"
    PLATFORM_NAME="macOS (Universal)"
elif [ "$IS_TERMUX" -eq 1 ]; then
    if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        FILE="MovieBox_Termux_arm64.tar.gz"
        PLATFORM_NAME="Android Termux (arm64)"
    else
        log_error "Unsupported Termux architecture ($ARCH). Only arm64/aarch64 is hosted. Use 'cargo install moviebox-tui'."
        exit 1
    fi
elif [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        FILE="MovieBox_Linux_x64.tar.gz"
        PLATFORM_NAME="Linux (x86_64)"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        FILE="MovieBox_Linux_arm64.tar.gz"
        PLATFORM_NAME="Linux (arm64)"
    else
        log_error "Unsupported Linux architecture ($ARCH). Only x86_64 and arm64 are supported."
        exit 1
    fi
else
    log_error "Unsupported Operating System ($OS)."
    exit 1
fi

resolve_version() {
    if [ -n "$VERSION" ]; then
        TARGET_VERSION="$VERSION"
        return 0
    fi

    local release_header
    release_header=$(curl -fsSI "https://github.com/$REPO/releases/latest") || {
        log_error "Failed to contact GitHub for latest release."
        return 1
    }
    TARGET_VERSION=$(printf "%s" "$release_header" | grep -i '^location:' | awk -F '/' '{print $NF}' | tr -d '\r\n')
    if [ -z "$TARGET_VERSION" ]; then
        log_error "Could not resolve latest release version from GitHub."
        return 1
    fi
}

TARGET_VERSION=""
run_spinner "[1/4] Checking environment & resolving version" resolve_version
log_success "[1/4] Environment ready ($PLATFORM_NAME • $TARGET_VERSION)"

if [ -n "$CUSTOM_DIR" ]; then
    INSTALL_DIR="$CUSTOM_DIR"
elif [ "$IS_TERMUX" -eq 1 ]; then
    INSTALL_DIR="$PREFIX/bin"
else
    INSTALL_DIR="$DEFAULT_INSTALL_DIR"
fi

APP_PATH="$INSTALL_DIR/$BIN_NAME"

EXISTING_BIN=$(command -v "$BIN_NAME" 2>/dev/null || true)
if [ -n "$EXISTING_BIN" ] && [ -x "$EXISTING_BIN" ]; then
    CURRENT_VERSION=$("$EXISTING_BIN" --version 2>/dev/null | awk '{print $2}' || true)
    CURRENT_VERSION=${CURRENT_VERSION:-unknown}

    if [ "v$CURRENT_VERSION" = "$TARGET_VERSION" ] && [ "$FORCE" -eq 0 ]; then
        if [ "$IS_TTY" -eq 1 ] && [ "$DRY_RUN" -eq 0 ]; then
            printf "\n  %b%s%b %s\n" "$C_BOLD_YELLOW" "ℹ" "$C_RESET" "MovieBox-TUI $TARGET_VERSION is already installed at $EXISTING_BIN."
            printf "  Choose an action: [1] Reinstall  [2] Uninstall  [3] Exit: "
            read -r user_choice || user_choice="3"
            case "$user_choice" in
                1)
                    log_step "Proceeding with reinstall..."
                    ;;
                2)
                    do_uninstall
                    ;;
                *)
                    log_success "No changes made. Exiting."
                    exit 0
                    ;;
            esac
        else
            log_success "MovieBox-TUI $TARGET_VERSION is already installed. Use --force to reinstall."
            exit 0
        fi
    fi
fi

if [ "$DRY_RUN" -eq 1 ]; then
    log_success "[Dry Run] Target package: $FILE"
    log_success "[Dry Run] Target install directory: $APP_PATH"
    log_success "[Dry Run] All preflight checks passed."
    exit 0
fi

TMP_DIR=$(mktemp -d)

URL="https://github.com/$REPO/releases/download/$TARGET_VERSION/$FILE"
CHECKSUM_URL="https://github.com/$REPO/releases/download/$TARGET_VERSION/SHA256SUMS"

download_files() {
    curl -fSL "$URL" -o "$TMP_DIR/$FILE" && \
    curl -fsSL "$CHECKSUM_URL" -o "$TMP_DIR/SHA256SUMS"
}

run_spinner "[2/4] Downloading $FILE" download_files
log_success "[2/4] Downloaded $FILE"

verify_checksum() {
    local expected_sha
    expected_sha=$(awk -v file="$FILE" '$2 == file {print $1}' "$TMP_DIR/SHA256SUMS")
    if [ -z "$expected_sha" ]; then
        return 1
    fi

    local actual_sha=""
    if command -v sha256sum >/dev/null 2>&1; then
        actual_sha=$(sha256sum "$TMP_DIR/$FILE" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual_sha=$(shasum -a 256 "$TMP_DIR/$FILE" | awk '{print $1}')
    elif command -v openssl >/dev/null 2>&1; then
        actual_sha=$(openssl dgst -sha256 "$TMP_DIR/$FILE" | awk '{print $NF}')
    else
        return 1
    fi

    [ "$actual_sha" = "$expected_sha" ]
}

run_spinner "[3/4] Verifying SHA256 checksum" verify_checksum
log_success "[3/4] Cryptographic checksum verified"

install_binary() {
    tar -xzf "$TMP_DIR/$FILE" -C "$TMP_DIR"
    if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
        return 1
    fi

    mkdir -p "$INSTALL_DIR" 2>/dev/null || true
    if [ -w "$INSTALL_DIR" ] || [ -w "$(dirname "$INSTALL_DIR")" ]; then
        mkdir -p "$INSTALL_DIR"
        install -m 755 "$TMP_DIR/$BIN_NAME" "$APP_PATH" 2>/dev/null || {
            cp "$TMP_DIR/$BIN_NAME" "$APP_PATH"
            chmod 755 "$APP_PATH"
        }
    elif command -v sudo >/dev/null 2>&1; then
        sudo mkdir -p "$INSTALL_DIR"
        sudo install -m 755 "$TMP_DIR/$BIN_NAME" "$APP_PATH" 2>/dev/null || {
            sudo cp "$TMP_DIR/$BIN_NAME" "$APP_PATH"
            sudo chmod 755 "$APP_PATH"
        }
    else
        INSTALL_DIR="$HOME/.local/bin"
        APP_PATH="$INSTALL_DIR/$BIN_NAME"
        mkdir -p "$INSTALL_DIR"
        cp "$TMP_DIR/$BIN_NAME" "$APP_PATH"
        chmod 755 "$APP_PATH"
    fi
}

run_spinner "[4/4] Installing binary to $INSTALL_DIR" install_binary
log_success "[4/4] Binary installed to $APP_PATH"

SHELL_MODIFIED=""
if [ "$NO_MODIFY_PATH" -eq 0 ]; then
    if ! echo "$PATH" | tr ':' '\n' | grep -q "^$INSTALL_DIR$"; then
        CURRENT_SHELL=$(basename "${SHELL:-bash}")
        RC_FILE=""
        case "$CURRENT_SHELL" in
            zsh)
                RC_FILE="$HOME/.zshrc"
                ;;
            bash)
                if [ -f "$HOME/.bashrc" ]; then
                    RC_FILE="$HOME/.bashrc"
                elif [ -f "$HOME/.bash_profile" ]; then
                    RC_FILE="$HOME/.bash_profile"
                else
                    RC_FILE="$HOME/.bashrc"
                fi
                ;;
            fish)
                RC_FILE="$HOME/.config/fish/config.fish"
                ;;
        esac

        if [ -n "$RC_FILE" ]; then
            mkdir -p "$(dirname "$RC_FILE")"
            if [ -f "$RC_FILE" ] && grep -q "$INSTALL_DIR" "$RC_FILE"; then
                :
            else
                if [ "$CURRENT_SHELL" = "fish" ]; then
                    printf "\nfish_add_path %s\n" "$INSTALL_DIR" >> "$RC_FILE"
                else
                    printf "\nexport PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR" >> "$RC_FILE"
                fi
                SHELL_MODIFIED="$RC_FILE"
            fi
        fi
    fi
fi

PLAYER_DETECTED=""
if command -v mpv >/dev/null 2>&1; then
    PLAYER_DETECTED="mpv"
elif command -v iina >/dev/null 2>&1 || command -v iina-cli >/dev/null 2>&1; then
    PLAYER_DETECTED="IINA"
elif command -v vlc >/dev/null 2>&1; then
    PLAYER_DETECTED="VLC"
fi

printf "\n"
printf "%b┌────────────────────────────────────────────────────────────┐%b\n" "$C_BOLD_CYAN" "$C_RESET"
printf "%b│%b  %b✔ MovieBox-Tui %s successfully installed!%b%*s%b│%b\n" \
    "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD_GREEN" "$TARGET_VERSION" "$C_RESET" \
    $(( 28 - ${#TARGET_VERSION} )) "" "$C_BOLD_CYAN" "$C_RESET"
printf "%b│%b                                                            %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
printf "%b│%b  • Binary:   %b%-45s%b %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD" "$APP_PATH" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"

if [ -n "$PLAYER_DETECTED" ]; then
    printf "%b│%b  • Player:   %b%-45s%b %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_GREEN" "$PLAYER_DETECTED (ready)" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
else
    printf "%b│%b  • Player:   %b%-45s%b %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_YELLOW" "None found (mpv / VLC / IINA required)" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
fi

if [ -n "$SHELL_MODIFIED" ]; then
    printf "%b│%b  • Shell:    %bPATH added to %-31s%b %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_CYAN" "$SHELL_MODIFIED" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
fi

printf "%b│%b                                                            %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
printf "%b│%b  To start streaming:                                       %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
printf "%b│%b    %b$ moviebox-tui%b                                         %b│%b\n" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD_CYAN" "$C_RESET"
printf "%b└────────────────────────────────────────────────────────────┘%b\n\n" "$C_BOLD_CYAN" "$C_RESET"

if [ -z "$PLAYER_DETECTED" ]; then
    log_warn "No media player detected. Please install mpv, VLC, or IINA for video playback."
fi

if [ -n "$SHELL_MODIFIED" ]; then
    printf "  %bℹ%b Run %b'source %s'%b or restart your terminal to reload PATH.\n\n" "$C_BOLD_CYAN" "$C_RESET" "$C_BOLD" "$SHELL_MODIFIED" "$C_RESET"
fi
