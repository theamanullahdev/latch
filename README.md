# Latch

A small app for Linux Mint (Cinnamon) that switches three risky things on and off: **Wine**, **xdotool** and the **firewall**.

I am building this right now. It is early. Things will break, and I will change things a lot.

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
- It installs its own pieces from Settings: the helper, a menu entry, the applet and the desklet. A clean Mint should be enough.

## The password rule

Making your machine safer should be free. Making it riskier should ask for a password.

| Switch   | Safer (no password)   | Riskier (password) |
|----------|-----------------------|--------------------|
| Wine     | lock                  | unlock             |
| xdotool  | disable               | enable             |
| Firewall | turn on               | turn off           |

The password prompt comes from polkit, so it is the normal system dialog. One password covers a short time, so changing a few things at once asks once.

## Where it is today

Early prototype. Be careful.

What I have tested, and how:

- **The root helper**, on a fake root (a throwaway user namespace with stand-in `wine`, `xdotool` and `ufw`). Install, all six jobs, bad input and uninstall all pass. Run it yourself: `extras/scripts/e2e.sh helper`.
- **The real window**, on that same fake root. I flipped switches, ran the Test buttons, installed and removed the helper from Settings, and used search.
- **The `.deb`**: I unpacked it and ran the app and helper that are inside it on the fake root. The polkit policy also passes polkit's own validator.
- **On my real desktop**: the themes (and remembering the choice), the menu entry, the panel applet and the desklet. The Wine test runs real Wine 9 with a sample program.

What I have **not** tested yet: the real polkit password prompt. My test setup has to stub it, so the first real flip on a real install is still ahead of me. 
Also tested on the fake root and my real desktop:

- **Timed unlock** for Wine and xdotool. Pick a delay under the switch (1 minute to 1 hour). The state line counts down and it locks itself at zero. Locking is the safe direction, so it never needs a password. If you close Latch while a timer runs, it locks early. If a timer ran out while Latch was closed, it locks on the next start. An unlock cannot be forgotten.
- **Custom app name and icon**, in Settings. The name shows in the window, the menu entry, the panel applet and the desklet. Any image works as an icon. A bad file is rejected.
- Installed applets and desklets update themselves when the app updates.

Not done yet: a Windows-style "run this one program with Wine unlocked" button, and per-app rules.

## Try it

You need Rust and the GTK3 development files.

```bash
sudo apt install libgtk-3-dev libglib2.0-dev
git clone https://github.com/theamanullahdev/latch.git
cd latch
cargo run -p latch-app
```

To open a page directly: `cargo run -p latch-app -- settings` (or `wine`, `xdotool`, `firewall`).

The first time you flip a switch, Latch installs a small root helper. That asks for your admin password once. You can remove it again in Settings.

## How it works, short version

The app runs as you. The few things that need root are done by a tiny helper that only knows six jobs (`wine-enable`, `wine-disable` and so on). Polkit decides which of those jobs need a password. The helper takes no input, so there is nothing to inject into.

More detail is in [lubalab.md](lubalab.md). Design notes are in [docs/design.md](docs/design.md) and the plan is in [docs/prd.md](docs/prd.md).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
