//! Settings persistence: a small plain-text file in the user's config folder.

use std::path::PathBuf;

use super::{factory_memories, Mem, ToneMem};
use crate::audio::{TEMPO_MAX, TEMPO_MIN};

pub struct Saved {
    pub memories: Vec<Mem>,
    pub tones: Vec<ToneMem>,
    pub memory: u8,
    pub sens: u8,
    pub guide: u8,
    /// unsaved panel settings at exit
    pub current: Option<Mem>,
    pub edited: bool,
    pub backlight: bool,
}

impl Default for Saved {
    fn default() -> Self {
        Saved {
            memories: factory_memories(),
            tones: vec![ToneMem::default(); 10],
            memory: 1,
            sens: 8,
            guide: 2,
            current: None,
            edited: false,
            backlight: false,
        }
    }
}

fn path() -> Option<PathBuf> {
    let env = |k| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
    }?;
    Some(base.join("kadence").join("settings.txt"))
}

fn mem_fields(m: &Mem) -> String {
    let s = m.sliders;
    format!(
        "{} {} {} {} {} {} {} {} {} {} {} {}",
        m.tempo, m.pattern_mode as u8, m.beat1, m.beat2, m.pattern, m.voice, s[0], s[1], s[2], s[3], s[4], m.repeats
    )
}

fn parse_mem(f: &[&str]) -> Option<Mem> {
    let n = |i: usize| f.get(i)?.parse::<u16>().ok();
    let b = |i: usize| n(i).map(|v| v.min(255) as u8);
    Some(Mem {
        tempo: n(0)?.clamp(TEMPO_MIN, TEMPO_MAX),
        pattern_mode: n(1)? != 0,
        beat1: b(2)?.clamp(1, 9),
        beat2: b(3)?.min(9),
        pattern: b(4)?.clamp(1, 30),
        // memories keep voice 4 on disk; `apply_mem` plays it as the nearest voice offered
        voice: b(5)?.clamp(1, 4),
        sliders: [b(6)?.min(100), b(7)?.min(100), b(8)?.min(100), b(9)?.min(100), b(10)?.min(100)],
        repeats: n(11)?.min(255),
    })
}

pub fn load() -> Saved {
    let mut s = Saved::default();
    let Some(text) = path().and_then(|p| std::fs::read_to_string(p).ok()) else {
        return s;
    };
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| f.get(i).and_then(|v| v.parse::<usize>().ok());
        match f.first().copied() {
            Some("memory") => s.memory = num(1).unwrap_or(1).clamp(1, 50) as u8,
            Some("sens") => s.sens = num(1).unwrap_or(8).clamp(1, 16) as u8,
            Some("guide") => s.guide = num(1).unwrap_or(2).clamp(1, 3) as u8,
            Some("edited") => s.edited = num(1) == Some(1),
            Some("light") => s.backlight = num(1) == Some(1),
            Some("cur") => s.current = parse_mem(&f[1..]),
            Some("m") => {
                if let (Some(i), Some(m)) = (num(1), parse_mem(f.get(2..).unwrap_or(&[]))) {
                    if (1..=50).contains(&i) {
                        s.memories[i - 1] = m;
                    }
                }
            }
            Some("t") => {
                if let (Some(i), Some(note), Some(pitch)) = (num(1), num(2), num(3)) {
                    if (1..=10).contains(&i) {
                        s.tones[i - 1] = ToneMem { note: note.clamp(36, 95) as u8, pitch: pitch.clamp(438, 445) as u16 };
                    }
                }
            }
            _ => {}
        }
    }
    s
}

pub fn save(s: &Saved) {
    let Some(p) = path() else { return };
    let mut out = String::from("# Kadence settings\n");
    out += &format!("memory {}\nsens {}\nguide {}\nedited {}\nlight {}\n", s.memory, s.sens, s.guide, s.edited as u8, s.backlight as u8);
    if let Some(c) = &s.current {
        out += &format!("cur {}\n", mem_fields(c));
    }
    for (i, m) in s.memories.iter().enumerate() {
        out += &format!("m {} {}\n", i + 1, mem_fields(m));
    }
    for (i, t) in s.tones.iter().enumerate() {
        out += &format!("t {} {} {}\n", i + 1, t.note, t.pitch);
    }
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&p, out) {
        eprintln!("kadence: cannot save settings to {}: {e}", p.display());
    }
}
