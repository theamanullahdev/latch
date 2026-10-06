const Applet = imports.ui.applet;
const PopupMenu = imports.ui.popupMenu;
const St = imports.gi.St;
const Clutter = imports.gi.Clutter;
const Gio = imports.gi.Gio;
const GLib = imports.gi.GLib;
const Cairo = imports.cairo;
const ByteArray = imports.byteArray;

const HELPER_DIR = "/usr/libexec/latch";
const SAFE = [0.435, 0.682, 0.310];
const WARN = [0.878, 0.627, 0.188];

/* Name from Latch settings (~/.config/latch/config, `name=`). Default Latch. */
function appName() {
    try {
        let [ok, data] = GLib.file_get_contents(GLib.get_user_config_dir() + "/latch/config");
        let m = ok && ByteArray.toString(data).match(/^name=(.+)$/m);
        return m ? m[1].trim() : "Latch";
    } catch (e) { return "Latch"; }
}

/* risky(on): exposed when true */
const TOGGLES = [
    { id: "wine", name: "Wine", bin: "/usr/bin/wine", risky: on => on },
    { id: "xdotool", name: "xdotool", bin: "/usr/bin/xdotool", risky: on => on },
    { id: "firewall", name: "Firewall", risky: on => !on },
];

/* null = unknown. Same rules as latch-core/probe.rs */
function probe(t) {
    try {
        if (t.id === "firewall") {
            let [ok, data] = GLib.file_get_contents("/etc/ufw/ufw.conf");
            let m = ok && String.fromCharCode.apply(null, data).match(/^ENABLED=(\w+)/m);
            return m ? m[1].toLowerCase() === "yes" : null;
        }
        let info = Gio.File.new_for_path(t.bin).query_info("unix::mode", Gio.FileQueryInfoFlags.NONE, null);
        return (info.get_attribute_uint32("unix::mode") & 1) !== 0;
    } catch (e) { return null; }
}

function runHelper(t, on, done) {
    let path = `${HELPER_DIR}/${t.id}-${on ? "enable" : "disable"}`;
    try {
        let p = new Gio.Subprocess({
            argv: ["pkexec", path],
            flags: Gio.SubprocessFlags.STDERR_SILENCE | Gio.SubprocessFlags.STDOUT_SILENCE
        });
        p.init(null);
        p.wait_async(null, (proc, res) => { try { proc.wait_finish(res); } catch (e) {} done(); });
    } catch (e) { done(); }
}

class LatchApplet extends Applet.Applet {
    constructor(metadata, orientation, panelHeight, instanceId) {
        super(orientation, panelHeight, instanceId);
        this._state = {};
        this._items = {};
        this._syncing = false;
        this._timer = 0;

        this._area = new St.DrawingArea({ y_align: Clutter.ActorAlign.CENTER, reactive: false });
        this._area.connect("repaint", area => this._repaint(area));
        this.actor.add_child(this._area);
        this._resize();

        this.menuManager = new PopupMenu.PopupMenuManager(this);
        this.menu = new Applet.AppletPopupMenu(this, orientation);
        this.menuManager.addMenu(this.menu);
        this._buildMenu();

        this._refresh();
        this._timer = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 5, () => {
            this._refresh();
            return GLib.SOURCE_CONTINUE;
        });
    }

    _buildMenu() {
        this._head = new PopupMenu.PopupMenuItem("", { reactive: false });
        this.menu.addMenuItem(this._head);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        for (let t of TOGGLES) {
            let item = new PopupMenu.PopupSwitchMenuItem(t.name, false);
            item.connect("toggled", (it, value) => this._onToggled(t, value));
            this._items[t.id] = item;
            this.menu.addMenuItem(item);
        }
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this._open = new PopupMenu.PopupMenuItem("Open");
        this._open.connect("activate", () => GLib.spawn_command_line_async("latch"));
        this.menu.addMenuItem(this._open);
    }

    _onToggled(t, value) {
        if (this._syncing) return;
        runHelper(t, value, () => this._refresh());
    }

    _refresh() {
        this._syncing = true;
        let exposed = 0;
        for (let t of TOGGLES) {
            let on = probe(t);
            this._state[t.id] = on;
            this._items[t.id].setSensitive(on !== null);
            if (on !== null) {
                this._items[t.id].setToggleState(on);
                if (t.risky(on)) exposed++;
            }
        }
        this._syncing = false;
        this._exposed = exposed;
        let text = exposed ? `${exposed} exposed` : "All safe";
        let name = appName();
        this._head.label.set_text(`${name}  ·  ${text}`);
        this._open.label.set_text(`Open ${name}`);
        this.set_applet_tooltip(`${name}: ${text}`);
        this._area.queue_repaint();
    }

    _resize() {
        let size = 20;
        try { size = this.getPanelIconSize(St.IconType.SYMBOLIC); } catch (e) {}
        this._area.set_size(size, size);
    }

    on_panel_icon_size_changed() { this._resize(); }

    on_applet_clicked() {
        this._refresh();
        this.menu.toggle();
    }

    on_applet_removed_from_panel() {
        if (this._timer) GLib.source_remove(this._timer);
        this._timer = 0;
    }

    /* Ring: closed green = safe. Gap + amber = exposed. */
    _repaint(area) {
        let cr = area.get_context();
        let [w, h] = area.get_surface_size();
        let risky = this._exposed > 0;
        let col = risky ? WARN : SAFE;
        let lw = Math.max(2, Math.round(w / 8));
        let r = Math.min(w, h) / 2 - lw / 2 - 0.5;
        let gap = risky ? 0.9 : 0;
        let top = -Math.PI / 2;

        cr.setLineWidth(lw);
        cr.setLineCap(Cairo.LineCap.ROUND);
        cr.setSourceRGBA(col[0], col[1], col[2], 1);
        if (risky) cr.arc(w / 2, h / 2, r, top + gap / 2, top + 2 * Math.PI - gap / 2);
        else cr.arc(w / 2, h / 2, r, 0, 2 * Math.PI);
        cr.stroke();

        cr.arc(w / 2, h / 2, r * 0.38, 0, 2 * Math.PI);
        cr.fill();
        cr.$dispose();
    }
}

function main(metadata, orientation, panelHeight, instanceId) {
    return new LatchApplet(metadata, orientation, panelHeight, instanceId);
}
