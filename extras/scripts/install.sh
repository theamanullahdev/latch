#!/usr/bin/env bash
# Build and install Latch for your own user. Works on Debian/Ubuntu/Mint/Kali, Arch/Manjaro,
# Fedora and openSUSE. Run it from a clone of the repo:
#
#   extras/scripts/install.sh
#
# Options:
#   --yes          do not ask questions
#   --no-run       do not start Latch at the end
#   --prefix DIR   install into DIR/bin (default: ~/.local)
#   --skip-deps    do not install system packages (you did it already)
#   --allow-root   allow running as root (for containers and CI only)
#   --uninstall    remove what this script installed
set -euo pipefail

YES=0 RUN=1 SKIP_DEPS=0 ALLOW_ROOT=0 UNINSTALL=0
PREFIX="$HOME/.local"
while [ $# -gt 0 ]; do
  case "$1" in
    --yes|-y) YES=1 ;;
    --no-run) RUN=0 ;;
    --skip-deps) SKIP_DEPS=1 ;;
    --allow-root) ALLOW_ROOT=1 ;;
    --uninstall) UNINSTALL=1 ;;
    --prefix) PREFIX=${2:?--prefix needs a folder}; shift ;;
    -h|--help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1 (try --help)" >&2; exit 2 ;;
  esac
  shift
done

say() { printf '\n==> %s\n' "$*"; }
die() { printf '\nerror: %s\n' "$*" >&2; exit 1; }

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
[ -f "$ROOT/Cargo.toml" ] && [ -d "$ROOT/crates/latch-app" ] || die "run this from a clone of the Latch repository"
export PATH="$HOME/.cargo/bin:$PATH"

if [ "$(id -u)" -eq 0 ] && [ "$ALLOW_ROOT" -ne 1 ]; then
  die "do not run this as root. Run it as your normal user. It asks for sudo only when it needs it."
fi
SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  SUDO="sudo"
fi

confirm() {
  [ "$YES" -eq 1 ] && return 0
  [ -r /dev/tty ] || die "no terminal to ask on. Run again with --yes."
  local answer
  read -r -p "$1 [y/N] " answer < /dev/tty
  [[ "$answer" =~ ^[Yy] ]]
}

# ---------- uninstall ----------
if [ "$UNINSTALL" -eq 1 ]; then
  say "Removing Latch (installed under $PREFIX)"
  confirm "Remove Latch and the things it set up for your user?" || exit 0
  LATCH="$PREFIX/bin/latch"
  if [ -x "$LATCH" ]; then
    for piece in menu applet desklet; do "$LATCH" setup "$piece" remove 2>/dev/null || true; done
    "$LATCH" setup helper remove || echo "(the system helper was not removed: see the message above)"
  fi
  rm -f "$PREFIX/bin/latch" "$PREFIX/bin/latch-helper"
  cat <<'EOF'

Done. Two things Latch does NOT undo, on purpose:
  1. Wine / xdotool that are still locked. Unlock them:
       for f in wine wine64 xdotool ydotool ydotoold wtype dotool dotoold; do
         p=$(readlink -f /usr/bin/$f) && [ -e "$p" ] && sudo chmod 755 "$p"; done
  2. Your settings and caches. Delete them if you want:
       rm -rf ~/.config/latch ~/.local/share/latch ~/.cache/latch
EOF
  exit 0
fi

# ---------- which distro? ----------
ID_ALL=""
if [ -r /etc/os-release ]; then
  # shellcheck disable=SC1091
  . /etc/os-release
  ID_ALL=" ${ID:-} ${ID_LIKE:-} "
fi
PM=""
case "$ID_ALL" in
  *" arch "*|*" manjaro "*|*" endeavouros "*) PM=pacman ;;
  *" debian "*|*" ubuntu "*|*" kali "*|*" linuxmint "*) PM=apt ;;
  *" fedora "*|*" rhel "*|*" centos "*) PM=dnf ;;
  *" suse "*|*" opensuse "*|*" sles "*) PM=zypper ;;
esac

PACKAGES=""
case "$PM" in
  apt)
    POLKIT=policykit-1
    if apt-cache show pkexec >/dev/null 2>&1; then POLKIT=pkexec; fi
    PACKAGES="build-essential pkg-config curl ca-certificates libgtk-3-dev libglib2.0-dev $POLKIT" ;;
  pacman) PACKAGES="base-devel pkgconf curl gtk3 glib2 polkit" ;;
  dnf)    PACKAGES="gcc make pkgconf-pkg-config curl gtk3-devel glib2-devel polkit" ;;
  zypper) PACKAGES="gcc make pkg-config curl ca-certificates gtk3-devel glib2-devel polkit" ;;
esac

MIN_RUST=$(grep -m1 '^rust-version' "$ROOT/Cargo.toml" | cut -d'"' -f2)
MIN_RUST=${MIN_RUST:-1.85}
case "$MIN_RUST" in *.*.*) ;; *) MIN_RUST="$MIN_RUST.0" ;; esac

say "Latch installer"
echo "Distro:   ${PRETTY_NAME:-unknown}"
echo "Install:  $PREFIX/bin/latch  (and latch-helper next to it)"
if [ "$SKIP_DEPS" -eq 1 ]; then
  echo "Packages: skipped (--skip-deps)"
elif [ -n "$PM" ]; then
  echo "Packages: $PACKAGES  (with $PM, using sudo)"
else
  die "I do not know this distro's package manager. Install GTK3 development files, glib, pkg-config, a C compiler, curl and polkit yourself, then run again with --skip-deps. See the README."
fi
echo "Rust:     $MIN_RUST or newer (a user-level copy is installed with rustup if yours is older)"
confirm "Go ahead?" || exit 0

# ---------- system packages ----------
if [ "$SKIP_DEPS" -ne 1 ]; then
  say "Installing system packages"
  # shellcheck disable=SC2086
  case "$PM" in
    apt)
      export DEBIAN_FRONTEND=noninteractive
      $SUDO apt-get update
      $SUDO apt-get install -y --no-install-recommends $PACKAGES ;;
    pacman) $SUDO pacman -S --needed --noconfirm $PACKAGES \
              || die "pacman failed. If it says a file was not found, run 'sudo pacman -Syu' first and try again." ;;
    dnf)    $SUDO dnf install -y $PACKAGES ;;
    zypper) $SUDO zypper --non-interactive install $PACKAGES ;;
  esac
fi

# ---------- Rust ----------
version_ge() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n1)" = "$2" ]; }
rust_ok() {
  command -v cargo >/dev/null 2>&1 || return 1
  local have
  have=$(cargo --version 2>/dev/null | awk '{print $2}') || return 1
  [ -n "$have" ] && version_ge "$have" "$MIN_RUST"
}
if ! rust_ok; then
  say "Rust $MIN_RUST or newer is needed (found: $(cargo --version 2>/dev/null || echo none))"
  if command -v rustup >/dev/null 2>&1; then
    rustup default stable
  else
    confirm "Install Rust for your user with rustup (https://rustup.rs)?" || die "Rust is needed to build Latch."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  fi
  # shellcheck disable=SC1091
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  rust_ok || die "could not get Rust $MIN_RUST or newer"
fi

# ---------- build and install ----------
say "Building Latch (a few minutes the first time)"
cd "$ROOT"
cargo build --release --workspace --locked

say "Installing into $PREFIX/bin"
install -Dm755 target/release/latch "$PREFIX/bin/latch"
install -Dm755 target/release/latch-helper "$PREFIX/bin/latch-helper"
"$PREFIX/bin/latch" setup menu install || echo "(could not add the menu entry. You can still run $PREFIX/bin/latch)"

# ---------- things worth telling the user ----------
say "Done"
case ":$PATH:" in
  *":$PREFIX/bin:"*) ;;
  *) echo "Add $PREFIX/bin to your PATH to type 'latch':  export PATH=\"$PREFIX/bin:\$PATH\"" ;;
esac
# Cinnamon and GNOME have the password dialog built in. Other desktops need a separate agent.
NEEDS_AGENT=1
case "${XDG_CURRENT_DESKTOP:-}" in *Cinnamon*|*GNOME*) NEEDS_AGENT=0 ;; esac
[ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] || NEEDS_AGENT=0
if [ "$NEEDS_AGENT" -eq 1 ] && ! pgrep -f -i 'polkit.*agent|polkit-gnome|polkit-mate|polkit-kde|lxpolkit|hyprpolkitagent|xfce-polkit' >/dev/null 2>&1; then
  echo
  echo "Heads up: no polkit agent seems to be running. Latch needs one to ask for your password."
  case "$PM" in
    pacman) echo "  Arch:   sudo pacman -S polkit-gnome   (or hyprpolkitagent on Hyprland), and start it with your session." ;;
    apt)    echo "  Debian: sudo apt install policykit-1-gnome   (or lxpolkit / mate-polkit), and start it with your session." ;;
    dnf)    echo "  Fedora: sudo dnf install polkit-gnome   and start it with your session." ;;
    *)      echo "  Install a polkit authentication agent for your desktop and start it with your session." ;;
  esac
fi
echo
echo "Latch is in $PREFIX/bin. The first time you flip a switch it asks for your password to set up its small root helper."
echo "Remove it later with:  extras/scripts/install.sh --uninstall"

if [ "$RUN" -eq 1 ]; then
  if [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then
    echo; echo "Starting Latch..."
    exec "$PREFIX/bin/latch"
  else
    echo "No graphical session here, so I did not start it. Run 'latch' from your desktop."
  fi
fi
