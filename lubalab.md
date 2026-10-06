# لب لباب

Map of repo. Read this, not the whole tree. Edit a line on any major change.

## Tree
```
Cargo.toml        workspace
lubalab.md        this map
crates/           rust code
  latch-core/     Toggle, Direction, password rules. no gtk
  latch-helper/   root binary. one file, 6 installed names `<toggle>-<enable|disable>`
  latch-app/      gtk3 gui, bin name `latch`
    src/ui/       window, sidebar, topbar (own headerbar), statusbar, panel (+test row), orb (animated badge), settings, state
    src/ui/autolock.rs timed unlock: 1 s ticker, flips switch off at 0. deadline saved in config. lock_pending() on window close
    src/branding.rs custom name + icon: config, windows, menu entry refresh
    src/backend.rs pkexec call, off ui thread. helper missing = setup::install first
    src/setup.rs  system helper install/remove (pkexec latch-helper install)
    src/menu.rs   ~/.local menu entry + icon
    src/cinnamon.rs applet + desklet enable/disable (gsettings), files embedded. refresh_installed() at start: rewrite stale ~/.local copies, ReloadXlet over dbus
    src/selftest.rs per-toggle live test (wine --version + cmd.exe, xdotool getmouselocation, ufw)
    src/fonts.rs  Bricolage Grotesque + Geist Mono embedded, to ~/.cache/latch/fonts
    src/theme/    base.css (always: spacing, state colors) + latch.css (palette, Latch mode only) + Mode switch
                  GTK3 quirk: later provider beats earlier one, no matter the specificity. Do not set `label {color}` in latch.css.
    src/config.rs ~/.config/latch/config (key=value: theme, name, icon, autolock_*, deadline_*). applet/desklet read `name=` in js
docs/             prd.md, design.md
extras/
  cinnamon/       applet latch@amanullah, desklet latch-desklet@amanullah (js, embedded in binary)
  fonts/          OFL ttf + licenses
  packaging/      polkit policy, .desktop, icons
  scripts/        pack.sh (make .deb into target/deb/), e2e.sh (fake-root tests: helper | gui | deb)
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
