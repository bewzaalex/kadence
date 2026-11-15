//! Disk cache for the rendered panel background: drawing it takes a second or more (its soft
//! shadows are large blurs), loading it back takes milliseconds.
//!
//! One file per window size, raw premultiplied RGBA behind a small header. The key is a hash of
//! the artwork and the size, so a new version of the panel never loads a stale picture.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use resvg::tiny_skia::{IntSize, Pixmap};

const MAGIC: &[u8; 8] = b"KDBG0001";
/// Sizes kept on disk (each is a few MB); the least recently written are removed.
const KEEP: usize = 3;

pub fn key(doc: &str, w: u32, h: u32) -> u64 {
    let mut s = DefaultHasher::new();
    (env!("CARGO_PKG_VERSION"), doc, w, h).hash(&mut s);
    s.finish()
}

fn file(w: u32, h: u32) -> Option<PathBuf> {
    Some(crate::dirs::cache()?.join(format!("background-{w}x{h}.bin")))
}

pub fn load(key: u64, w: u32, h: u32) -> Option<Pixmap> {
    let data = std::fs::read(file(w, h)?).ok()?;
    let (head, pixels) = data.split_at_checked(16)?;
    if &head[..8] != MAGIC || head[8..] != key.to_le_bytes() {
        return None;
    }
    Pixmap::from_vec(pixels.to_vec(), IntSize::from_wh(w, h)?)
}

/// Best effort: a failed write only means the next start draws the background again.
pub fn save(key: u64, pm: &Pixmap) {
    let (Some(dir), Some(path)) = (crate::dirs::cache(), file(pm.width(), pm.height())) else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let mut data = Vec::with_capacity(16 + pm.data().len());
    data.extend_from_slice(MAGIC);
    data.extend_from_slice(&key.to_le_bytes());
    data.extend_from_slice(pm.data());
    // write then rename, so a half-written file is never loaded
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, data).is_err() || std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    prune(&dir);
}

fn prune(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<_> = entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("background-"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse(f.0));
    for (_, p) in files.into_iter().skip(KEEP) {
        let _ = std::fs::remove_file(p);
    }
}
