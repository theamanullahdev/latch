# Changelog

All notable changes to Latch. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [Semantic Versioning](https://semver.org/). Versions below 1.0.0 are early prototypes.

## [Unreleased]

### Added
- Arch Linux packaging: a `PKGBUILD` and an AUR publish workflow (it stays idle until AUR credentials are added).
- CI builds and tests on a real Arch Linux container, and builds the Arch package.
- Wayland groundwork: the "xdotool" switch also covers `ydotool`, `ydotoold`, `wtype`, `dotool` and `dotoold`.
- Terminal commands: `latch status`, `latch status --json` (for Waybar on Hyprland) and `latch set <toggle> <on|off>`.
- README sections for Arch and Hyprland, issue templates, and Dependabot for the pinned GitHub Actions.

- `extras/scripts/install.sh`: one command to build and install on Debian-family, Arch, Fedora and openSUSE systems. It installs build packages, gets a new enough Rust (Ubuntu and Mint ship one that is too old), builds, installs into `~/.local` and adds a menu entry. `--uninstall` removes it again.
- CI runs the installer on clean containers of Ubuntu 24.04, Linux Mint 22, Debian 13, Kali, Arch, Fedora and openSUSE, and installs the `.deb` on the four Debian-family ones.
- `latch setup <helper|menu|applet|desklet> <install|remove>`: what the Settings buttons do, from a terminal.
- The minimum Rust version (1.85) is declared in `Cargo.toml`.

### Changed
- The `.deb` now only *suggests* `wine` and `ufw`. Before, apt installed them by default (Recommends), which pulled a whole Wine into a small panel app.
- Settings, `latch setup helper remove` and the helper itself now refuse to remove a helper that a package manager installed. Only a helper that Latch installed itself can be removed that way.
- A switch that covers several files now reads as unlocked if any one of them can still be run.
- Latch theme asks GTK for its built-in dark variant too, so it stays dark where the Adwaita-dark theme is not installed.

## [0.1.0] - 2026-10-21

First public pre-release.

### Added
- GTK3 app for Linux Mint (Cinnamon) with one page per switch: **Wine**, **xdotool** and **Firewall**.
- Polkit-gated root helper. Making things safer needs no password. Making things riskier does.
- A live **Test** button under each switch that runs the real tool and compares the result with the switch.
- **Timed unlock** for Wine and xdotool (1 minute to 1 hour). Closing the app, or restarting after the time ran out, locks early.
- Cinnamon **panel applet** and **desktop desklet**.
- Own Latch theme (no blue, no purple) and an option to follow the system theme.
- Custom app name and icon in Settings.
- Search box that jumps to the matching page.
- Settings can install and remove the root helper, the menu entry, the applet and the desklet.
- A signed apt repository (`sudo apt install latch`) and a `.deb` with SHA-256 checksums on each release.
- End-to-end test script that runs the real helper, the real window and the `.deb` on a throwaway fake root.

### Known limits
- Tested on Linux Mint 22.3 (Cinnamon, x86_64) only.
- No Wayland, no other desktops, no other distros.

[0.1.0]: https://github.com/theamanullahdev/latch/releases/tag/v0.1.0
