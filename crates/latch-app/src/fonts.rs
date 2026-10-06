//! Fonts live inside the binary: Bricolage Grotesque (UI), Geist Mono (output).
//! Written to the user cache, then added to fontconfig for this process.

use gtk::glib;
use std::{ffi::{c_void, CString}, fs, path::Path};

#[link(name = "fontconfig")]
extern "C" {
    fn FcConfigAppFontAddFile(config: *mut c_void, file: *const u8) -> i32;
}

const FONTS: [(&str, &[u8]); 4] = [
    ("BricolageGrotesque-Regular.ttf", include_bytes!("../../../extras/fonts/BricolageGrotesque-Regular.ttf")),
    ("BricolageGrotesque-Bold.ttf", include_bytes!("../../../extras/fonts/BricolageGrotesque-Bold.ttf")),
    ("GeistMono-Regular.ttf", include_bytes!("../../../extras/fonts/GeistMono-Regular.ttf")),
    ("GeistMono-Bold.ttf", include_bytes!("../../../extras/fonts/GeistMono-Bold.ttf")),
];

pub fn load() {
    let dir = glib::user_cache_dir().join("latch").join("fonts");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    for (name, bytes) in FONTS {
        let path = dir.join(name);
        let stale = fs::metadata(&path).map_or(true, |m| m.len() != bytes.len() as u64);
        if stale && fs::write(&path, bytes).is_err() {
            continue;
        }
        add(&path);
    }
}

fn add(path: &Path) {
    let Ok(file) = CString::new(path.to_string_lossy().as_bytes()) else { return };
    unsafe {
        FcConfigAppFontAddFile(std::ptr::null_mut(), file.as_ptr().cast());
    }
}
