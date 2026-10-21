# لب لباب

Map of repo. Read this, not the whole tree. Edit a line on any major change.

## Tree
```
Cargo.toml        workspace
lubalab.md        this map
CHANGELOG.md      keep-a-changelog. release notes come from the matching section
.github/          workflows: distros.yml (installer on 7 distro containers + .deb on 4), ci.yml (test + clippy + Arch container build), release.yml (v* tag -> .deb, release, apt repo), aur.yml (v* tag -> AUR). dependabot.yml, ISSUE_TEMPLATE/
crates/           rust code
  latch-core/     Toggle, Direction, password rules. no gtk
  latch-helper/   root binary. one file, 6 installed names `<toggle>-<enable|disable>`
  latch-app/      gtk3 gui, bin name `latch`
    src/ui/       window, sidebar, topbar (own headerbar), statusbar, panel (+test row), orb (animated badge), settings, state
    src/ui/autolock.rs timed unlock: 1 s ticker, flips switch off at 0. deadline saved in config. lock_pending() on window close
    src/branding.rs custom name + icon: config, windows, menu entry refresh
    src/cli.rs    `latch status [--json]` (Waybar), `latch set <toggle> <on|off>`. no gtk, runs before the window
    src/backend.rs pkexec call, off ui thread. helper missing = setup::install first
    src/setup.rs  system helper install/remove (pkexec latch-helper install)
    src/menu.rs   ~/.local menu entry + icon
    src/cinnamon.rs applet + desklet enable/disable (gsettings), files embedded. refresh_installed() at start: rewrite stale ~/.local copies, ReloadXlet over dbus
    src/selftest.rs per-toggle live test (wine --version + cmd.exe, xdotool getmouselocation, ufw)
    src/fonts.rs  Bricolage Grotesque + Geist Mono embedded, to ~/.cache/latch/fonts
    src/theme/    base.css (always: spacing, state colors) + latch.css (palette, Latch mode only) + Mode switch
                  GTK3 quirk: later provider beats earlier one, no matter the specificity. Do not set `label {color}` in latch.css.
    src/config.rs ~/.config/latch/config (key=value: theme, name, icon, autolock_*, deadline_*). applet/desklet read `name=` in js
docs/             prd.md, design.md, screenshots/ (README images, taken on the fake root)
extras/
  cinnamon/       applet latch@amanullah, desklet latch-desklet@amanullah (js, embedded in binary)
  fonts/          OFL ttf + licenses
  packaging/      polkit policy, .desktop, icons, apt-index.html (landing page of the apt repo), arch/PKGBUILD
  scripts/        install.sh (newcomer installer: apt/pacman/dnf/zypper, rustup if cargo < MSRV, installs to ~/.local), apt-repo.sh (signed apt repo with apt-ftparchive), pack.sh (make .deb into target/deb/), e2e.sh (fake-root tests: helper | gui | deb)
  packaging/scripts/  postinst, postrm (refresh icon + menu cache)
```

## Flow
```
latch (user) --pkexec /usr/libexec/latch/<toggle>-<dir>--> helper (root)
helper: chmod 700/755 wine, xdotool. `ufw --force enable` / `ufw disable`
polkit gates by helper path (annotate exec.path). safer = yes, riskier = auth_admin_keep
state read w/o root: exec bit, /etc/ufw/ufw.conf  (core/probe.rs)
applet, desklet read state alone (same probe in js), flip via same pkexec helpers
helper `install`: copies self to /usr/libexec/latch/{6 actions + latch-helper}, writes polkit policy
```

## Rules
- Root: max 3 folders. Now: crates, docs, extras.
- GTK3 not GTK4. Mint themes are GTK3.
- Applet, desklet = JS only. Thin. Different uuid each: Cinnamon looks up by uuid alone.
- Self-contained: nothing read from this machine. Fonts, js, policy, icon embedded. Other users start empty.
- `latch <page>` opens a page: wine, xdotool, firewall, settings.
- Polkit ids: `org.latch.<toggle>.<enable|disable>`. Policy: extras/packaging/polkit.
- Helper takes no args. Action = own file name. Never path, never shell string.
- Safer = no password. Riskier = password.

## Dev
- `cargo dev` run gui from source.
- `extras/scripts/pack.sh` make .deb. .deb lands in target/deb/.
- First switch flip or Settings > System helper installs root helper (admin prompt).
- Build deps: libgtk-3-dev, libglib2.0-dev, cargo-deb.
- E2E: `extras/scripts/e2e.sh helper|gui [page]|deb FILE [page]`. `E2E_CONFIG='autolock_wine=60\n'` seeds the fake home config. Prints fake wine mode, config, menu entry after the GUI closes. bwrap user ns, fake wine/xdotool/ufw, pkexec stub. Real system untouched. Polkit prompt itself not covered.
- Screenshots: `latch <page>` then gnome-screenshot -w. xdotool clicks need window activated first.

## Release
- Install for users: apt repo `https://theamanullahdev.github.io/latch` (suite stable, component main, amd64). Pages serves the `gh-pages` branch.
- New version: bump `version` in Cargo.toml, add `## [X.Y.Z] - date` to CHANGELOG.md, commit, then `git tag -a vX.Y.Z -m "..."` and `git push origin vX.Y.Z`.
- release.yml checks tag == Cargo version, runs test + clippy, builds, makes .deb + SHA256SUMS, creates the GitHub Release (pre-release while 0.x), and adds the .deb to the signed apt repo.
- Signing key: secret `APT_GPG_PRIVATE_KEY` in the repo. Private backup lives outside the repo (see local memory). Lose it = rotate the key and every user re-adds it.
- Actions are pinned by commit SHA. Bump them on purpose, not with `@v7`.
- Built on ubuntu-24.04 (glibc 2.39), so Mint 22+ and Ubuntu 24.04+ only.
- Arch: PKGBUILD in extras/packaging/arch. aur.yml pushes it to the AUR on a tag, but only if repo secret `AUR_SSH_PRIVATE_KEY` exists. One-time setup (needs a human): make an AUR account, add an SSH public key to it, put the private key in that secret. First push to `ssh://aur@aur.archlinux.org/latch.git` creates the package.
- CI `arch` job: builds + tests in archlinux:base-devel and runs makepkg on the PKGBUILD (it builds the last tagged release tarball).
- Input tools covered by the "xdotool" switch: see INPUT_TOOLS in latch-core/src/probe.rs. Add new tools there only.
- GTK3 quirk: cross-provider CSS, later provider wins regardless of specificity.
- MSRV is `rust-version` in Cargo.toml (1.85, from indexmap/hashbrown). install.sh reads it from there. Ubuntu/Mint apt cargo is 1.75, too old.
- Helper installed by `latch-helper install` writes the marker `.self-installed`. Package-installed helpers have none, so uninstall refuses (Settings, CLI and helper all agree).
- Test installer pieces locally without touching the system: `XDG_DATA_HOME=/tmp/x GSETTINGS_BACKEND=memory extras/scripts/install.sh --yes --no-run --skip-deps --prefix /tmp/x`.
