# Changelog

All notable changes to Latch. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [Semantic Versioning](https://semver.org/). Versions below 1.0.0 are early prototypes.

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
