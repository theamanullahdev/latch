//! Live check per toggle: run the real tool, report if it launched.

use gtk::gio;
use latch_core::Toggle;
use std::{
    io::ErrorKind,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct Outcome {
    /// Tool usable right now. Compared with the switch by the caller.
    pub works: bool,
    pub lines: Vec<String>,
}

pub async fn run(t: Toggle) -> Outcome {
    gio::spawn_blocking(move || check(t)).await.unwrap_or(Outcome {
        works: false,
        lines: vec!["test crashed".into()],
    })
}

fn check(t: Toggle) -> Outcome {
    match t {
        Toggle::Wine => wine(),
        Toggle::Xdotool => xdotool(),
        Toggle::Firewall => firewall(),
    }
}

fn wine() -> Outcome {
    let mut lines = Vec::new();
    let version = exec(Command::new("/usr/bin/wine").arg("--version"), 10);
    lines.push(report("wine --version", &version));
    if version.is_ok() {
        let prefix = gtk::glib::user_cache_dir().join("latch").join("wine");
        let mut sample = Command::new("/usr/bin/wine");
        sample
            .args(["cmd", "/c", "echo", "latch-ok"])
            .env("WINEPREFIX", prefix)
            .env("WINEDEBUG", "-all")
            .env("WINEDLLOVERRIDES", "mscoree,mshtml=");
        let ran = exec(&mut sample, 90).and_then(|o| {
            if o.contains("latch-ok") { Ok("ran".into()) } else { Err("no output".into()) }
        });
        lines.push(report("sample .exe (cmd.exe)", &ran));
        return Outcome { works: ran.is_ok(), lines };
    }
    Outcome { works: false, lines }
}

fn xdotool() -> Outcome {
    let out = exec(Command::new("/usr/bin/xdotool").arg("getmouselocation"), 10);
    Outcome { works: out.is_ok(), lines: vec![report("xdotool getmouselocation", &out)] }
}

fn firewall() -> Outcome {
    let service = exec(Command::new("systemctl").args(["is-active", "ufw"]), 10);
    let conf = latch_core::probe::is_on(Toggle::Firewall);
    let lines = vec![
        report("service ufw", &service),
        format!("ufw.conf ENABLED: {}", conf.map_or("unknown", |on| if on { "yes" } else { "no" })),
    ];
    Outcome { works: conf == Some(true) && service.is_ok(), lines }
}

fn report(what: &str, res: &Result<String, String>) -> String {
    match res {
        Ok(out) => format!("{what}: {out}"),
        Err(e) => format!("{what}: {e}"),
    }
}

/// First stdout line on success. Short reason on failure.
fn exec(cmd: &mut Command, secs: u64) -> Result<String, String> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => "blocked (permission denied)".to_string(),
            ErrorKind::NotFound => "not installed".to_string(),
            _ => e.to_string(),
        })?;
    let end = Instant::now() + Duration::from_secs(secs);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < end => thread::sleep(Duration::from_millis(40)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("timed out".into());
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let first = |b: &[u8]| String::from_utf8_lossy(b).lines().next().unwrap_or("").trim().to_string();
    if out.status.success() {
        Ok(first(&out.stdout))
    } else {
        let err = first(&out.stderr);
        Err(if err.is_empty() { format!("exit {}", out.status.code().unwrap_or(-1)) } else { err })
    }
}
