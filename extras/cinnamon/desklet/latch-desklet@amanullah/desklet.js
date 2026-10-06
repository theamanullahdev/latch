const Desklet = imports.ui.desklet;
const St = imports.gi.St;
const Gio = imports.gi.Gio;
const GLib = imports.gi.GLib;
const ByteArray = imports.byteArray;

const HELPER_DIR = "/usr/libexec/latch";

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
    { id: "wine", name: "Wine", bin: "/usr/bin/wine", text: ["Locked", "Unlocked"], risky: on => on },
    { id: "xdotool", name: "xdotool", bin: "/usr/bin/xdotool", text: ["Disabled", "Enabled"], risky: on => on },
    { id: "firewall", name: "Firewall", text: ["Off", "Active"], risky: on => !on },
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

class LatchDesklet extends Desklet.Desklet {
    constructor(metadata, deskletId) {
        super(metadata, deskletId);
        this._rows = {};
        this._timer = 0;
        this._build();
        this._refresh();
        this._timer = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 5, () => {
            this._refresh();
            return GLib.SOURCE_CONTINUE;
        });
    }

    _build() {
        this._card = new St.BoxLayout({ vertical: true, style_class: "latch-card" });
        let head = new St.BoxLayout({ style_class: "latch-head" });
        this._title = new St.Label({ text: "", style_class: "latch-title" });
        head.add(this._title, { expand: true, y_fill: false });
        this._summary = new St.Label({ text: "", style_class: "latch-summary" });
        head.add(this._summary, { y_fill: false });
        this._card.add_actor(head);

        for (let t of TOGGLES) {
            let button = new St.Button({ style_class: "latch-row", reactive: true, can_focus: true, x_fill: true });
            let row = new St.BoxLayout();
            let dot = new St.Bin({ style_class: "latch-dot", y_align: St.Align.MIDDLE });
            let name = new St.Label({ text: t.name, style_class: "latch-name" });
            let state = new St.Label({ text: "", style_class: "latch-state" });
            row.add(dot, { y_fill: false });
            row.add(name, { expand: true, y_fill: false });
            row.add(state, { y_fill: false });
            button.set_child(row);
            button.connect("clicked", () => this._flip(t));
            this._rows[t.id] = { dot, state, button };
            this._card.add_actor(button);
        }
        this.setContent(this._card);
    }

    _flip(t) {
        let on = probe(t);
        if (on === null) return;
        this._rows[t.id].button.reactive = false;
        runHelper(t, !on, () => { this._rows[t.id].button.reactive = true; this._refresh(); });
    }

    _refresh() {
        this._title.set_text(appName().toUpperCase());
        let exposed = 0;
        for (let t of TOGGLES) {
            let on = probe(t);
            let row = this._rows[t.id];
            if (on === null) {
                row.state.set_text("Not found");
                row.dot.style_class = "latch-dot";
                continue;
            }
            let risky = t.risky(on);
            if (risky) exposed++;
            row.state.set_text(t.text[on ? 1 : 0]);
            row.dot.style_class = risky ? "latch-dot latch-risky" : "latch-dot latch-safe";
        }
        this._summary.set_text(exposed ? `${exposed} exposed` : "All safe");
        this._summary.style_class = exposed ? "latch-summary latch-risky-text" : "latch-summary";
    }

    on_desklet_removed() {
        if (this._timer) GLib.source_remove(this._timer);
        this._timer = 0;
    }
}

function main(metadata, deskletId) {
    return new LatchDesklet(metadata, deskletId);
}
