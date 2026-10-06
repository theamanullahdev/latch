# Design

This file is the visual source of truth. Add screenshots and sketches under `docs/design/`.
Status: first draft. Fill in as the prototype grows.

## Principles
- Native when `Use system theme` is on. Own palette otherwise.
- Calm. Few colors. One clear state per switch.
- Alive. Small animations that show a change happened.

## Type
- UI: Bricolage Grotesque. Output and logs: Geist Mono. Both OFL, embedded.

## Badge (orb)
- Ring closed + green = safe. Ring opens, spins, amber = exposed.
- Glyph per toggle: wine glass (liquid level), cursor, shield + check.
- Filled glyph = the active thing. Liquid sloshes while exposed.

## Hard choices (author)
- No blue. No purple. Default palette: charcoal, warm gray, amber accent. Green = safe, red = alert.
- Corners: radius 4px. Not sharp. Not pill.
- Layout: one workspace screen. No stacked cards, no endless scroll.
- Setting `Theme`: Latch (default, forced dark base + palette) or System (Cinnamon theme).

## Layout
```
+--------------------------------------------+
| nav / title                 [ search     ] |
+---------+----------------------------------+
| sidebar |  panel (one section at a time)   |
| Wine    |                                  |
| xdotool |                                  |
| Firewall|                                  |
| Settings|                                  |
+---------+----------------------------------+
| status bar: state summary                  |
+--------------------------------------------+
```
New features = new sidebar entry. Panel never scrolls the whole app.

## Theme adoption (Cinnamon)
- Use plain GTK3 widgets. Do not hard-code colors or fonts.
- Colors come from the active GTK theme (Mint-Y, Mint-L, or any custom theme).
- Use theme named colors in CSS only: `@theme_bg_color`, `@theme_fg_color`, `@theme_selected_bg_color`.
- Icons come from the active icon theme by name, symbolic where possible.
- Follow `org.cinnamon.desktop.interface` for theme and icon changes, live, without restart.
- Dark and light must both work. Test with Mint-Y, Mint-Y-Dark, and one third-party theme.
- Use `xapp` widgets where they help (status icon, preferences style).

## Main window
- Top bar: nav, title, search.
- Left sidebar: Wine, xdotool, Firewall, Settings.
- Panel: badge, name, state, switch, then Test row (button, output, verdict).
- Title bar is ours (headerbar), so it follows the Latch theme.
- Locked state shows a lock icon. Unlocked shows an open lock.
- Timed unlock shows a countdown ring on the card.
- Bottom status bar: summary, such as "All locked".

(Add mockup: `docs/design/main-window.png`)

## States
| State       | Look                                   |
|-------------|----------------------------------------|
| Safe        | accent color, closed lock              |
| Risky (on)  | warning color, open lock               |
| Working     | spinner on the switch, switch disabled |
| Needs auth  | system polkit dialog, app dims         |
| Error       | inline message on the card, no popup   |

## Animations
- Switch flip: 150 ms ease.
- Card state change: color cross-fade, 200 ms.
- Countdown ring: smooth, updates each second.
- All animations respect the GTK "enable animations" setting and the app's own toggle.

## Applet (panel)
- Icon shows overall state (all safe, or something is on).
- Click opens a small popup with the same switches.
- Written in JavaScript (Cinnamon requires it). Talks to the app over D-Bus.

## Desklet (desktop)
- A small card widget with the same switches and state.
- Size and transparency are settings.
- Same D-Bus link as the applet.

## Customization
- App name: user text. Changes the window title, the menu entry, the applet label.
- App icon: pick from the icon theme or choose a file.

## References to collect
- Cinnamon applet and desklet examples from the Spices site.
- Mint-Y and Mint-L screenshots.
- Any desktop widget style the author likes. Add links here.
