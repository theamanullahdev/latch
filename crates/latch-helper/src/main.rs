//! Root helper. Run by pkexec.
//! Installed as `<toggle>-<enable|disable>`: fixed job, args ignored.
//! Named `latch-helper`: `install` / `uninstall` the system copies.

use latch_core::{probe, Direction, Toggle};
use std::{env, fs, io, os::unix::fs::PermissionsExt, path::Path, process::Command};

const UFW: &str = "/usr/sbin/ufw";
const POLICY: &str = "/usr/share/polkit-1/actions/org.latch.policy";
const POLICY_BODY: &str = include_str!("../../../extras/packaging/polkit/org.latch.policy");

fn parse(name: &str) -> Option<(Toggle, Direction)> {
    let (toggle, dir) = name.rsplit_once('-')?;
    let dir = match dir {
        "enable" => Direction::Enable,
        "disable" => Direction::Disable,
        _ => return None,
    };
    Some((Toggle::from_id(toggle)?, dir))
}

/// Open = 755. Locked = 700.
fn set_open(path: &Path, open: bool) -> io::Result<()> {
    let mode = if open { 0o755 } else { 0o700 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

fn run(t: Toggle, dir: Direction) -> io::Result<()> {
    let open = dir == Direction::Enable;
    match t {
        Toggle::Firewall => {
            let args: &[&str] = if open { &["--force", "enable"] } else { &["disable"] };
            let status = Command::new(UFW).args(args).status()?;
            status.success().then_some(()).ok_or_else(|| io::Error::other("ufw failed"))
        }
        _ => {
            let bins = probe::binaries(t);
            if bins.is_empty() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "not installed"));
            }
            bins.iter().try_for_each(|b| set_open(b, open))
        }
    }
}

/// Names of all installed copies, plus the admin one.
fn copies() -> Vec<String> {
    let actions = Toggle::ALL.iter().flat_map(|t| {
        [Direction::Enable, Direction::Disable].map(|d| format!("{}-{}", t.id(), d.id()))
    });
    actions.chain(["latch-helper".to_string()]).collect()
}

fn install() -> io::Result<()> {
    let me = env::current_exe()?;
    let dir = Path::new(latch_core::HELPER_DIR);
    fs::create_dir_all(dir)?;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o755))?;
    for name in copies() {
        let dest = dir.join(name);
        let _ = fs::remove_file(&dest);
        fs::copy(&me, &dest)?;
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
    }
    fs::write(POLICY, POLICY_BODY)?;
    fs::set_permissions(POLICY, fs::Permissions::from_mode(0o644))
}

fn uninstall() -> io::Result<()> {
    let dir = Path::new(latch_core::HELPER_DIR);
    for name in copies() {
        let _ = fs::remove_file(dir.join(name));
    }
    let _ = fs::remove_dir(dir);
    let _ = fs::remove_file(POLICY);
    Ok(())
}

fn fail(msg: String, code: i32) -> ! {
    eprintln!("latch-helper: {msg}");
    std::process::exit(code);
}

fn main() {
    let mut args = env::args();
    let exe = args.next().unwrap_or_default();
    let name = Path::new(&exe).file_name().and_then(|n| n.to_str()).unwrap_or("");
    if let Some((t, dir)) = parse(name) {
        run(t, dir).unwrap_or_else(|e| fail(format!("{}-{}: {e}", t.id(), dir.id()), 1));
        return;
    }
    let result = match (name, args.next().as_deref()) {
        ("latch-helper", Some("install")) => install(),
        ("latch-helper", Some("uninstall")) => uninstall(),
        _ => fail(format!("bad name '{name}'"), 2),
    };
    result.unwrap_or_else(|e| fail(e.to_string(), 1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(parse("wine-enable"), Some((Toggle::Wine, Direction::Enable)));
        assert_eq!(parse("firewall-disable"), Some((Toggle::Firewall, Direction::Disable)));
        assert_eq!(parse("wine"), None);
        assert_eq!(parse("rm-enable"), None);
        assert_eq!(parse("latch-helper"), None);
    }

    #[test]
    fn copies_cover_all_actions() {
        let c = copies();
        assert_eq!(c.len(), 7);
        assert!(c.contains(&"firewall-disable".to_string()));
        assert!(c.contains(&"latch-helper".to_string()));
    }

    #[test]
    fn chmod_open_lock() {
        let p = env::temp_dir().join("latch-test-bin");
        fs::write(&p, "x").unwrap();
        set_open(&p, false).unwrap();
        assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o700);
        set_open(&p, true).unwrap();
        assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o755);
        fs::remove_file(&p).unwrap();
    }
}
