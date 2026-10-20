# Latch

[![CI](https://github.com/theamanullahdev/latch/actions/workflows/ci.yml/badge.svg)](https://github.com/theamanullahdev/latch/actions/workflows/ci.yml)

A small app for Linux Mint (Cinnamon) that switches three risky things on and off: **Wine**, **xdotool** and the **firewall**.

I am building this right now. It is early. Things will break, and I will change things a lot.

<p align="center">
  <img src="docs/screenshots/wine.png" width="31%" alt="Wine page">
  <img src="docs/screenshots/firewall.png" width="31%" alt="Firewall page">
  <img src="docs/screenshots/settings.png" width="31%" alt="Settings page">
</p>

## Install

Latch is for **Linux Mint 22 and newer** (and Ubuntu 24.04 and newer), on x86_64 with the Cinnamon desktop. That is all I have tried it on.

**Option 1: apt (recommended).** Add the Latch repository once, then install and update like any other package.

```bash
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://theamanullahdev.github.io/latch/key.asc | sudo gpg --dearmor -o /etc/apt/keyrings/latch.gpg
echo "deb [signed-by=/etc/apt/keyrings/latch.gpg] https://theamanullahdev.github.io/latch stable main" | sudo tee /etc/apt/sources.list.d/latch.list
sudo apt update
sudo apt install latch
```

**Option 2: one file.** Download the `.deb` and its checksum from the [Releases](https://github.com/theamanullahdev/latch/releases) page.

```bash
sha256sum -c SHA256SUMS
sudo apt install ./latch_0.1.0-1_amd64.deb
```

**Option 3: build it yourself.** See [Build from source](#build-from-source).

Then start **Latch** from the app menu, or run `latch` in a terminal. In Settings you can add the panel applet and the desktop desklet.

Other distros: only Debian-based packages for now (Mint, Ubuntu). Arch and others are not packaged. Building from source should work anywhere GTK3 is available.

### Uninstall

```bash
sudo apt remove latch
sudo rm -f /etc/apt/sources.list.d/latch.list /etc/apt/keyrings/latch.gpg   # if you used option 1
```

Before you remove it, open Settings and press **Remove** on the applet and the desklet. Your own settings live in `~/.config/latch`, `~/.local/share/latch` and `~/.cache/latch`. Delete them if you want a full clean up.

**Important:** removing Latch does not undo what it did. If Wine or xdotool is locked at that moment, unlock them first. See the next section.

## Security: what Latch changes, and how to undo it

Latch changes system settings, so you should know exactly what it touches.

| Switch   | What it does                                                                        | Undo by hand |
|----------|-------------------------------------------------------------------------------------|--------------|
| Wine     | Lock = `chmod 700` on the Wine program files. Unlock = `chmod 755`.                 | `sudo chmod 755 "$(readlink -f /usr/bin/wine)"` and the same for `wine64` |
| xdotool  | Lock = `chmod 700` on `/usr/bin/xdotool`. Unlock = `chmod 755`.                     | `sudo chmod 755 "$(readlink -f /usr/bin/xdotool)"` |
| Firewall | `ufw --force enable` or `ufw disable`. It never adds, removes or edits your rules. | `sudo ufw enable` or `sudo ufw disable` |

To unlock everything in one go:

```bash
for f in wine wine64 xdotool; do p=$(readlink -f /usr/bin/$f) && [ -e "$p" ] && sudo chmod 755 "$p"; done
```

Good to know:

- A locked file can still be run by root. Locking only stops normal users, and the agents that run as you.
- If the `wine` or `xdotool` package is upgraded, the new files come back unlocked. Latch shows the real state, so you will see it.
- Making things safer (lock, disable, firewall on) never asks for a password. Making things riskier (unlock, enable, firewall off) does.

What gets installed on the machine:

- `/usr/bin/latch`, the app.
- `/usr/libexec/latch/`: seven small root programs. Six do one fixed job each (`wine-enable`, `firewall-disable` and so on) and take no input. The seventh, `latch-helper`, only installs or removes this set.
- `/usr/share/polkit-1/actions/org.latch.policy`: the rules for which job needs a password.
- A menu entry and an icon.
- In your home folder: `~/.config/latch`, `~/.local/share/latch`, `~/.cache/latch`, and (only if you press Install in Settings) the applet and desklet under `~/.local/share/cinnamon`.

Found a security problem? Please report it privately: open the **Security** tab of this repo and choose **Report a vulnerability**. Please do not post exploit details in a public issue.

## Why I am making this

I build Electron apps on Linux, and sometimes I need Wine to make a Windows `.exe`. Wine is great, but it also lets any `.exe` run on my machine. I do not trust random `.exe` files. So for a while I did something silly: I uninstalled Wine after every build and installed it again the next time.

That got old fast, so I wrote a few shell scripts that lock and unlock Wine ([wine-locker](https://github.com/theamanullahdev/wine-locker)). They flip the execute permission on the Wine binaries. They worked, but they were just scripts in a terminal.

Then a bigger problem showed up. I now let AI agents work on my computer. With a tool like `xdotool` an agent can move my mouse, type on my keyboard and click things, and nobody has to ask me first. I do not want that to be always on. I want to decide when an agent can touch my desktop, and when it cannot.

The same goes for the firewall. I want one place to see it and flip it.

So Latch is a control panel for exactly that: what my machine allows, and what it does not, without me asking first.

## What it is

- A GUI app first. The terminal is not the point.
- Made for Cinnamon. Written in Rust with GTK3.
- One window with a sidebar, one page per switch, and a status bar at the bottom.
- A panel applet and a desktop desklet, so you can see the state without opening anything.
- It has its own look (charcoal, warm gray, amber, no blue, no purple). You can switch to your system theme in Settings.
- Each switch has a **Test** button right under it. It runs the real tool, so you can see if the switch did what it says. For Wine it runs a tiny sample program.
- **Timed unlock** for Wine and xdotool: pick 1 minute to 1 hour and it locks itself again. If you close Latch while a timer runs, it locks early, so an unlock cannot be forgotten.
- A custom app name and icon, in Settings.
- It installs its own pieces from Settings: the helper, a menu entry, the applet and the desklet.

## The password rule

Making your machine safer should be free. Making it riskier should ask for a password.

| Switch   | Safer (no password)   | Riskier (password) |
|----------|-----------------------|--------------------|
| Wine     | lock                  | unlock             |
| xdotool  | disable               | enable             |
| Firewall | turn on               | turn off           |

The password prompt comes from polkit, so it is the normal system dialog. One password covers a short time, so changing a few things at once asks once.

## Where it is today

Early prototype (version 0.1.0, marked as a pre-release). Be careful.

- I have the `.deb` installed on my own Linux Mint 22.3 machine and I use it there.
- I tested the root helper, the whole window, the `.deb` and the timed unlock on a throwaway fake root (a user namespace with stand-in `wine`, `xdotool` and `ufw`). Run the helper checks yourself: `extras/scripts/e2e.sh helper`.
- The polkit rules pass polkit's own validator.
- I have **not** tried other distros, other Mint versions, other desktops, or Wayland.

Not done yet: a "run this one program with Wine unlocked" button, and per-app rules. See the [CHANGELOG](CHANGELOG.md) for what changed so far.

## Build from source

You need Rust and the GTK3 development files.

```bash
sudo apt install libgtk-3-dev libglib2.0-dev
git clone https://github.com/theamanullahdev/latch.git
cd latch
cargo run -p latch-app
```

To open a page directly: `cargo run -p latch-app -- settings` (or `wine`, `xdotool`, `firewall`).

The first time you flip a switch, Latch installs a small root helper. That asks for your admin password once. You can remove it again in Settings.

To make your own `.deb`:

```bash
cargo install cargo-deb
extras/scripts/pack.sh          # writes target/deb/
```

## How it works, short version

The app runs as you. The few things that need root are done by a tiny helper that only knows six jobs (`wine-enable`, `wine-disable` and so on). Polkit decides which of those jobs need a password. The helper takes no input, so there is nothing to inject into.

More detail is in [lubalab.md](lubalab.md). Design notes are in [docs/design.md](docs/design.md) and the plan is in [docs/prd.md](docs/prd.md).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
