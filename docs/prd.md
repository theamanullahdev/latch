# PRD: Latch (working name)

Status: draft. The name is a placeholder. Name and icon are user settings anyway.

## What it is
A GUI app for Linux Mint Cinnamon. It gives one place to switch risky things on and off:
wine, xdotool (agent control of the desktop), and the firewall.
It should feel good to use. It lives in the menu, in a panel applet, and on the desktop as a desklet.

## Who it is for
First, the author. Then other Linux Mint users who run Wine or let AI agents drive their PC.

## Goals
0. Self-contained. Works on a clean Mint. App installs its own pieces. No dependence on author machine.
1. GUI first. The CLI exists, but the GUI is the product.
2. Looks native on Cinnamon. Follows the user's GTK theme, icons, and dark or light mode.
3. Passwords only where it matters (see the table below).
4. One `.deb` installs everything: app, menu entry, applet, desklet, polkit rules.
5. Fast dev loop. `cargo dev` runs the app from source. No `.deb` per test.

## Non-goals (for now)
- Other distros or other desktops.
- Remote control of the toggles.
- Per-app wine rules.

## Toggles and password rules
| Toggle   | Turn off (safer)       | Turn on (riskier)        |
|----------|------------------------|--------------------------|
| Wine     | no password            | password                 |
| xdotool  | no password            | password                 |
| Firewall | password               | no password              |

Rule: making the system safer is free. Making it riskier asks for a password.
One password covers a short window, so changing many settings at once asks once.
Timed unlock ("wine on for 60 seconds, then lock") is kept from the old wine-locker.

## Surfaces
- Main window.
- Menu entry (`.desktop`).
- Cinnamon panel applet.
- Cinnamon desklet.
- CLI: `latch wine off`, `latch status`, and so on.

## Settings page (in the app)
- App name and icon (custom). Name flows to window, menu entry, applet, desklet.
- Install / remove: system helper, menu entry, panel applet, desktop desklet.
- Each toggle page has a live Test (real tool run, verdict vs switch).
- Theme: Latch or System. Default Latch. Saved in ~/.config/latch/config.
- Auto-lock delay per toggle (Wine, xdotool) lives on each page, under the switch.
- Animations on or off.

## Open questions
- Final name.
- Should disabling xdotool also kill running xdotool processes?
- Re-lock wine after `apt` updates (dpkg hook)?
- Firewall: do we manage individual rules, or only the on/off switch?

## Prototype scope (v0.1)
Main window with three switches. Real wine toggle through the helper and polkit. No applet yet.
