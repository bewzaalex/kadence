//! SVG sources for every part of the panel (port of design/mockup/panel.js).
//!
//! The case is split into a static background and small dynamic layers (LCD, knob,
//! button caps, slider caps, LEDs) so only what changes is re-rendered.

use std::f64::consts::PI;
use std::fmt::Write;

use crate::device::{Btn, Device, Edit, Mode};

pub const W: f64 = 610.0;
pub const H: f64 = 800.0;
pub const DIAL: (f64, f64) = (492.0, 524.0);
pub const SLIDERS: [f64; 6] = [70.0, 112.0, 154.0, 196.0, 238.0, 300.0];
pub const SL_TOP: f64 = 494.0;
pub const SL_BOT: f64 = 610.0;
pub const LCD_RECT: (f64, f64, f64, f64) = (60.0, 218.0, 250.0, 200.0);
pub const LEDS: [(f64, f64); 2] = [(113.0, 196.0), (237.0, 196.0)];
pub const MIC_RECT: (f64, f64, f64, f64) = (350.0, 575.0, 40.0, 40.0);

const PRINT: &str = "#c3c6c9";
const PRINT_SIZE: f64 = 8.5;

pub fn slider_cap_y(v: u8) -> f64 {
    SL_BOT - 14.0 - (v as f64 / 100.0) * (SL_BOT - SL_TOP - 28.0)
}

pub fn slider_value(y: f64) -> u8 {
    (((SL_BOT - 14.0 - y) / (SL_BOT - SL_TOP - 28.0)) * 100.0).round().clamp(0.0, 100.0) as u8
}

#[derive(Clone, Copy, PartialEq)]
pub enum Style {
    Plain,
    Boxed,
    Light,
    Single,
}

pub struct ButtonDef {
    pub id: Btn,
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
    pub label: &'static str,
    pub style: Style,
    pub size: f64,
    pub rim: bool,
}

const fn b(id: Btn, cx: f64, cy: f64, w: f64, h: f64, label: &'static str, style: Style, size: f64, rim: bool) -> ButtonDef {
    ButtonDef { id, cx, cy, w, h, label, style, size, rim }
}

pub const BUTTONS: [ButtonDef; 16] = [
    b(Btn::Power, 384.0, 322.0, 60.0, 34.0, "POWER", Style::Boxed, 9.6, true),
    b(Btn::Light, 458.0, 322.0, 60.0, 34.0, "LIGHT", Style::Plain, 9.6, false),
    b(Btn::Mute, 532.0, 322.0, 60.0, 34.0, "MUTE", Style::Plain, 9.6, false),
    b(Btn::Write, 384.0, 378.0, 60.0, 34.0, "WRITE", Style::Plain, 9.6, true),
    b(Btn::Memory, 458.0, 378.0, 60.0, 34.0, "MEMORY", Style::Plain, 8.8, false),
    b(Btn::Midi, 532.0, 378.0, 60.0, 34.0, "MIDI", Style::Plain, 9.6, false),
    b(Btn::Tempo, 75.0, 670.0, 60.0, 36.0, "TEMPO", Style::Plain, 9.6, false),
    b(Btn::Loop, 150.0, 670.0, 60.0, 36.0, "LOOP", Style::Plain, 9.6, false),
    b(Btn::Tune, 225.0, 670.0, 60.0, 36.0, "TUNE", Style::Boxed, 9.6, false),
    b(Btn::Coach, 300.0, 670.0, 60.0, 36.0, "COACH", Style::Light, 9.6, false),
    b(Btn::Beat, 75.0, 724.0, 60.0, 36.0, "BEAT/PATTERN", Style::Plain, 9.4, false),
    b(Btn::Voice, 150.0, 724.0, 60.0, 36.0, "VOICE", Style::Plain, 9.6, false),
    b(Btn::Pitch, 225.0, 724.0, 60.0, 36.0, "PITCH", Style::Boxed, 9.6, false),
    b(Btn::Trig, 300.0, 724.0, 60.0, 36.0, "TRIG/GUIDE", Style::Light, 9.4, false),
    b(Btn::Tap, 380.0, 697.0, 54.0, 90.0, "TAP", Style::Plain, 12.0, false),
    b(Btn::Start, 494.0, 697.0, 132.0, 90.0, "START/STOP", Style::Single, 13.0, false),
];

// ------------------------------------------------------------------ defs

const DEFS: &[(&str, &str)] = &[
    ("body", r##"<linearGradient id="body" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#85888b"/><stop offset="0.5" stop-color="#696c6f"/><stop offset="1" stop-color="#4f5254"/></linearGradient>"##),
    ("sheen", r##"<linearGradient id="sheen" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.10"/><stop offset="0.28" stop-color="#fff" stop-opacity="0.16"/><stop offset="0.6" stop-color="#fff" stop-opacity="0"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>"##),
    ("blur1", r##"<filter id="blur1" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="1"/></filter>"##),
    ("blur2", r##"<filter id="blur2" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="2"/></filter>"##),
    ("blur3", r##"<filter id="blur3" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur stdDeviation="3"/></filter>"##),
    ("blur4", r##"<filter id="blur4" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="4"/></filter>"##),
    // the same blurs for shapes as big as the case: a tight filter region (the blur reaches only
    // a few units past the shape, and blurring a 20% margin of the whole case is slow)
    ("caseBlur1", r##"<filter id="caseBlur1" x="-3%" y="-3%" width="106%" height="106%"><feGaussianBlur stdDeviation="1"/></filter>"##),
    ("caseBlur2", r##"<filter id="caseBlur2" x="-3%" y="-3%" width="106%" height="106%"><feGaussianBlur stdDeviation="2"/></filter>"##),
    ("caseBlur3", r##"<filter id="caseBlur3" x="-3%" y="-3%" width="106%" height="106%"><feGaussianBlur stdDeviation="3"/></filter>"##),
    ("glow", r##"<filter id="glow" x="-100%" y="-200%" width="300%" height="500%"><feGaussianBlur stdDeviation="3"/></filter>"##),
    ("hole", r##"<linearGradient id="hole" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#050506"/><stop offset="1" stop-color="#232427"/></linearGradient>"##),
    ("edgeHole", r##"<linearGradient id="edgeHole" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#2a2c2e"/><stop offset="0.5" stop-color="#55585b"/><stop offset="1" stop-color="#a7aaad"/></linearGradient>"##),
    ("edgeLit", r##"<linearGradient id="edgeLit" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ffffff" stop-opacity="0.55"/><stop offset="0.5" stop-color="#ffffff" stop-opacity="0.05"/><stop offset="1" stop-color="#000" stop-opacity="0.45"/></linearGradient>"##),
    ("chamferBig", r##"<linearGradient id="chamferBig" x1="0" y1="0" x2="0.6" y2="1"><stop offset="0" stop-color="#000" stop-opacity="0.5"/><stop offset="0.45" stop-color="#000" stop-opacity="0.12"/><stop offset="0.7" stop-color="#fff" stop-opacity="0.08"/><stop offset="1" stop-color="#fff" stop-opacity="0.3"/></linearGradient>"##),
    ("bevelV", r##"<linearGradient id="bevelV" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.38"/><stop offset="0.08" stop-color="#fff" stop-opacity="0.06"/><stop offset="0.9" stop-color="#000" stop-opacity="0.05"/><stop offset="1" stop-color="#000" stop-opacity="0.5"/></linearGradient>"##),
    ("bevelH", r##"<linearGradient id="bevelH" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff" stop-opacity="0.22"/><stop offset="0.1" stop-color="#fff" stop-opacity="0"/><stop offset="0.9" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.38"/></linearGradient>"##),
    ("capEdge", r##"<linearGradient id="capEdge" x1="0" y1="0" x2="0.3" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.28"/><stop offset="0.45" stop-color="#fff" stop-opacity="0.04"/><stop offset="1" stop-color="#000" stop-opacity="0.5"/></linearGradient>"##),
    ("capTop", r##"<linearGradient id="capTop" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.13"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>"##),
    ("knurlShade", r##"<linearGradient id="knurlShade" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.35"/><stop offset="0.45" stop-color="#fff" stop-opacity="0"/><stop offset="1" stop-color="#5a2a00" stop-opacity="0.55"/></linearGradient>"##),
    ("bump", r##"<radialGradient id="bump" cx="0.35" cy="0.3" r="0.75"><stop offset="0" stop-color="#ffe2a3"/><stop offset="0.5" stop-color="#ffab1e"/><stop offset="1" stop-color="#c46c00"/></radialGradient>"##),
    ("dimpleWall", r##"<linearGradient id="dimpleWall" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#9a5000" stop-opacity="0.8"/><stop offset="0.5" stop-color="#ffa30f" stop-opacity="0"/><stop offset="1" stop-color="#ffe7b0" stop-opacity="0.9"/></linearGradient>"##),
    ("black", r##"<linearGradient id="black" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#151617"/><stop offset="1" stop-color="#0c0c0d"/></linearGradient>"##),
    ("btnGrad", r##"<linearGradient id="btnGrad" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#303134"/><stop offset="0.6" stop-color="#1d1e20"/><stop offset="1" stop-color="#131314"/></linearGradient>"##),
    ("capGrad", r##"<linearGradient id="capGrad" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffc24a"/><stop offset="0.5" stop-color="#ffa514"/><stop offset="1" stop-color="#e88e06"/></linearGradient>"##),
    ("dialGrad", r##"<radialGradient id="dialGrad" cx="0.38" cy="0.33" r="0.8"><stop offset="0" stop-color="#ffbb36"/><stop offset="0.65" stop-color="#ffa310"/><stop offset="1" stop-color="#ec9004"/></radialGradient>"##),
    ("dimple", r##"<radialGradient id="dimple" cx="0.62" cy="0.68" r="0.7"><stop offset="0" stop-color="#ffc85a"/><stop offset="0.55" stop-color="#ffa30f"/><stop offset="1" stop-color="#d77a00"/></radialGradient>"##),
    ("lcdOff", r##"<radialGradient id="lcdOff" cx="0.5" cy="0.45" r="0.75"><stop offset="0" stop-color="#bcc3ab"/><stop offset="1" stop-color="#a1a993"/></radialGradient>"##),
    ("lcdOn", r##"<radialGradient id="lcdOn" cx="0.5" cy="0.45" r="0.75"><stop offset="0" stop-color="#ffc77e"/><stop offset="0.7" stop-color="#f6a24a"/><stop offset="1" stop-color="#df8530"/></radialGradient>"##),
    ("glass", r##"<linearGradient id="glass" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.16"/><stop offset="0.4" stop-color="#fff" stop-opacity="0.02"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>"##),
];

fn defs(ids: &[&str], paint_uri: Option<&str>) -> String {
    let mut o = String::from("<defs>");
    for (id, d) in DEFS {
        if ids.contains(id) {
            o += d;
        }
    }
    if let Some(uri) = paint_uri {
        let _ = write!(o, r#"<pattern id="paint" width="80" height="80" patternUnits="userSpaceOnUse"><image href="{uri}" width="80" height="80"/></pattern>"#);
    }
    o + "</defs>"
}

pub fn doc(body: &str, ids: &[&str], paint_uri: Option<&str>) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" font-family="Noto Sans" font-weight="bold">{}{}</svg>"#,
        defs(ids, paint_uri),
        body
    )
}

fn f1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn pts(p: &[(f64, f64)]) -> String {
    let mut o = String::new();
    for (i, (x, y)) in p.iter().enumerate() {
        if i > 0 {
            o.push(' ');
        }
        let _ = write!(o, "{},{}", f1(*x), f1(*y));
    }
    o
}

// ------------------------------------------------------------------ LCD

const GLYPHS: &[(char, &str)] = &[
    ('0', "abcdef"), ('1', "bc"), ('2', "abdeg"), ('3', "abcdg"), ('4', "bcfg"), ('5', "acdfg"),
    ('6', "acdefg"), ('7', "abc"), ('8', "abcdefg"), ('9', "abcdfg"), ('-', "g"), (' ', ""),
    ('A', "abcefg"), ('b', "cdefg"), ('C', "adef"), ('c', "deg"), ('d', "bcdeg"), ('E', "adefg"),
    ('F', "aefg"), ('G', "acdef"), ('h', "cefg"), ('H', "bcefg"), ('L', "def"), ('n', "ceg"),
    ('o', "cdeg"), ('P', "abefg"), ('q', "abcfg"), ('r', "eg"), ('S', "acdfg"), ('t', "defg"),
    ('u', "cde"), ('U', "bcdef"), ('y', "bcdfg"),
];

fn glyph(c: char) -> &'static str {
    GLYPHS.iter().find(|(g, _)| *g == c).map(|(_, m)| *m).unwrap_or("")
}

/// LCD ink: lit segments, ghost (unlit) segments, inverted text.
struct Ink {
    on: &'static str,
    gh: f64,
    inv: &'static str,
}

struct Lcd {
    o: String,
    ink: Ink,
}

const BIG: (f64, f64, f64) = (22.0, 38.0, 4.4);
const BIG_XS: [f64; 3] = [92.0, 118.0, 144.0];
const BIG_Y: f64 = 7.0;
const SM: (f64, f64, f64) = (11.0, 19.0, 2.5);
const MEM_XS: [f64; 2] = [22.0, 36.0];
const RIGHT_XS: [f64; 3] = [194.0, 208.0, 222.0];
const PIVOT: (f64, f64) = (125.0, 150.0);
pub const PEND_N: usize = 17;
const PEND_SPAN: f64 = 44.0;

fn hseg(x1: f64, x2: f64, y: f64, t: f64) -> [(f64, f64); 6] {
    let h = t / 2.0;
    [(x1, y), (x1 + h, y - h), (x2 - h, y - h), (x2, y), (x2 - h, y + h), (x1 + h, y + h)]
}
fn vseg(x: f64, y1: f64, y2: f64, t: f64) -> [(f64, f64); 6] {
    let h = t / 2.0;
    [(x, y1), (x + h, y1 + h), (x + h, y2 - h), (x, y2), (x - h, y2 - h), (x - h, y1 + h)]
}

impl Lcd {
    fn fill(&self, lit: bool) -> String {
        if lit { format!(r#"fill="{}""#, self.ink.on) } else { format!(r#"fill="{}" opacity="{}""#, self.ink.on, self.ink.gh) }
    }

    fn label(&mut self, txt: &str, x: f64, y: f64, on: bool, size: f64, anchor: &str) {
        if on {
            let _ = write!(self.o, r#"<text x="{x}" y="{y}" font-size="{size}" text-anchor="{anchor}" fill="{}">{txt}</text>"#, self.ink.on);
        }
    }
    fn lbl(&mut self, txt: &str, x: f64, y: f64, on: bool) {
        self.label(txt, x, y, on, 6.6, "start");
    }

    fn digit_mask(&mut self, x: f64, y: f64, (w, h, t): (f64, f64, f64), mask: &str, visible: bool) {
        let g = t * 0.18;
        let (xl, xr, yt, ym, yb) = (t / 2.0, w - t / 2.0, t / 2.0, h / 2.0, h - t / 2.0);
        let segs = [
            ('a', hseg(xl + g, xr - g, yt, t)),
            ('b', vseg(xr, yt + g, ym - g, t)),
            ('c', vseg(xr, ym + g, yb - g, t)),
            ('d', hseg(xl + g, xr - g, yb, t)),
            ('e', vseg(xl, ym + g, yb - g, t)),
            ('f', vseg(xl, yt + g, ym - g, t)),
            ('g', hseg(xl + g, xr - g, ym, t)),
        ];
        let _ = write!(self.o, r#"<g transform="translate({},{}) skewX(-7)">"#, f1(x), f1(y));
        for (k, p) in segs {
            let lit = visible && mask.contains(k);
            let f = self.fill(lit);
            let _ = write!(self.o, r#"<polygon {f} points="{}"/>"#, pts(&p));
        }
        self.o += "</g>";
    }

    fn digit(&mut self, x: f64, y: f64, size: (f64, f64, f64), c: char, visible: bool) {
        self.digit_mask(x, y, size, glyph(c), visible);
    }

    /// Right-aligned text into the given digit cells.
    fn digits(&mut self, xs: &[f64], y: f64, size: (f64, f64, f64), text: &str, visible: bool) {
        let chars: Vec<char> = text.chars().collect();
        let n = xs.len();
        for (i, x) in xs.iter().enumerate() {
            let k = (i + chars.len()) as isize - n as isize;
            let c = if k >= 0 { chars[k as usize] } else { ' ' };
            self.digit(*x, y, size, c, visible);
        }
    }

    fn pendulum(&mut self, idx: Option<usize>) {
        for i in 0..PEND_N {
            let a = (-PEND_SPAN + (2.0 * PEND_SPAN * i as f64) / (PEND_N - 1) as f64) * PI / 180.0;
            let (r0, r1, w0, w1) = (20.0, 90.0, 3.2, 1.3);
            let (sx, cy, px, py) = (a.sin(), -a.cos(), a.cos(), a.sin());
            let p = |r: f64, w: f64| (PIVOT.0 + sx * r + px * w, PIVOT.1 + cy * r + py * w);
            let poly = [p(r0, -w0), p(r1, -w1), p(r1 + 2.0, 0.0), p(r1, w1), p(r0, w0)];
            let f = self.fill(idx == Some(i));
            let _ = write!(self.o, r#"<polygon {f} points="{}"/>"#, pts(&poly));
        }
    }

    fn wedge(&mut self, x: f64, bottom: f64, level: u8) {
        for i in 0..5 {
            let hh = 3.0 + i as f64 * 2.8;
            let f = self.fill((i as u8) < level);
            let _ = write!(self.o, r#"<rect {f} x="{}" y="{}" width="3.6" height="{}"/>"#, f1(x + i as f64 * 4.6), f1(bottom - hh), f1(hh));
        }
    }

    fn note_icon(&mut self, kind: u8, x: f64, y: f64) {
        let on = self.ink.on;
        let head = |cx: f64, cy: f64| format!(r#"<ellipse fill="{on}" cx="{cx}" cy="{cy}" rx="2.6" ry="1.9" transform="rotate(-22 {cx} {cy})"/>"#);
        let stem = |cx: f64, cy: f64| format!(r#"<rect fill="{on}" x="{}" y="{}" width="0.9" height="10"/>"#, cx + 1.9, cy - 10.0);
        let flag = |cx: f64, cy: f64, n: u8| {
            let mut o = String::new();
            for i in 0..n {
                let _ = write!(o, r#"<path fill="{on}" d="M{},{} q3,2 3,5 q-0.8,-2.4 -3,-3.2z"/>"#, cx + 2.8, cy - 10.0 + i as f64 * 2.6);
            }
            o
        };
        let rest8 = |rx: f64, ry: f64| format!(r#"<path fill="{on}" d="M{rx},{ry} a1.2,1.2 0 1,1 1.6,1.1 l2.4,-1.1 l-2.6,7 h-1 l2.2,-5.6 l-2.6,0.6z"/>"#);
        let note = |cx: f64, cy: f64, flags: u8| head(cx, cy) + &stem(cx, cy) + &flag(cx, cy, flags);
        let s = match kind {
            1 => note(x + 4.0, y, 0),
            2 => rest8(x, y - 9.0) + &note(x + 9.0, y, 1),
            3 => rest8(x - 3.0, y - 9.0) + &note(x + 5.0, y, 2) + &rest8(x + 11.0, y - 9.0) + &note(x + 19.0, y, 2),
            _ => {
                let mut o = String::new();
                for i in 0..3 {
                    o += &head(x + i as f64 * 6.0, y);
                    o += &stem(x + i as f64 * 6.0, y);
                }
                let _ = write!(o, r#"<rect fill="{on}" x="{}" y="{}" width="12.9" height="1.6"/>"#, x + 1.9, y - 10.0);
                o
            }
        };
        self.o += &s;
        if kind == 4 {
            self.label("3", x + 8.0, y - 12.0, true, 5.0, "middle");
        }
    }

    fn sun(&mut self, cx: f64, cy: f64, on: bool) {
        let f = self.fill(on);
        let _ = write!(self.o, r#"<circle {f} cx="{cx}" cy="{cy}" r="2.4"/>"#);
        for i in 0..8 {
            let _ = write!(self.o, r#"<rect {f} x="{}" y="{}" width="1" height="2.2" transform="rotate({} {cx} {cy})"/>"#, cx - 0.5, cy - 6.0, i * 45);
        }
    }
}

const NOTE_NAMES: [&str; 12] = ["C", "C#", "d", "d#", "E", "F", "F#", "G", "G#", "A", "A#", "b"];

/// LCD layer: glass content in panel coordinates.
pub fn lcd(d: &Device) -> String {
    // while powering off the light and then the segments fade out
    let light = if d.backlight && d.power { d.light_level } else { 0.0 };
    let backlit = light > 0.5;
    let ink = if backlit { Ink { on: "#2b1606", gh: 0.07, inv: "#f6a24a" } } else { Ink { on: "#17180f", gh: 0.055, inv: "#b4bba2" } };
    let mut l = Lcd { o: String::new(), ink };
    let (lx, ly, lw, lh) = LCD_RECT;
    let _ = write!(l.o, r#"<g transform="translate({lx},{ly})"><rect x="0" y="0" width="{lw}" height="{lh}" fill="url(#lcdOff)"/>"#);
    if light > 0.0 {
        let _ = write!(l.o, r#"<rect x="0" y="0" width="{lw}" height="{lh}" fill="url(#lcdOn)" opacity="{light:.3}"/>"#);
    }

    if !d.power {
        l.digits(&BIG_XS, BIG_Y, BIG, "   ", false);
        l.pendulum(None);
    } else if d.ink_level < 1.0 {
        let _ = write!(l.o, r#"<g opacity="{:.3}">"#, d.ink_level);
        lcd_content(&mut l, d);
        l.o += "</g>";
    } else {
        lcd_content(&mut l, d);
    }
    l.o += "</g>";
    let _ = write!(l.o, r#"<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" fill="url(#glass)"/>"#);
    doc(&l.o, &["lcdOn", "lcdOff", "glass"], None)
}

fn lcd_content(l: &mut Lcd, d: &Device) {
    let hide = |e: Edit| d.edit == Some(e) && !d.blink;
    let mode = d.mode;

    // top-left: memory / sens
    if mode == Mode::Coach {
        l.lbl("TRIG", 6.0, 12.0, true);
        l.lbl("SENS", 6.0, 30.0, true);
        l.digits(&MEM_XS, 20.0, SM, &d.sens.to_string(), !hide(Edit::Sens));
        let _ = write!(l.o, r#"<rect fill="{}" x="50" y="31" width="27" height="9" rx="1"/><text fill="{}" x="63.5" y="38" font-size="6.6" text-anchor="middle">COACH</text>"#, l.ink.on, l.ink.inv);
        l.label("TRIG", 6.0, 52.0, true, 5.6, "start");
        let f = l.fill(d.trig);
        let _ = write!(l.o, r#"<circle {f} cx="14" cy="59" r="2.6"/>"#);
    } else {
        l.lbl("MEMORY", 6.0, 15.0, !hide(Edit::Memory) || mode == Mode::Tune);
        let n = match (mode, d.edit) {
            (_, Some(Edit::Write)) => d.write_target,
            (Mode::Tune, _) => d.tone_memory,
            _ => d.memory,
        };
        let vis = !(hide(Edit::Memory) || hide(Edit::Write)) && !(d.write_flashing() && !d.fast_blink);
        l.digits(&MEM_XS, 20.0, SM, &n.to_string(), vis);
        if d.edited && mode != Mode::Tune {
            l.label("EDIT", 40.0, 47.0, true, 5.6, "start");
        }
    }

    let arrow = |l: &mut Lcd| {
        let _ = write!(l.o, r#"<path fill="{}" d="M60,2 l3,4 l3,-4z"/>"#, l.ink.on);
    };
    match mode {
        Mode::Metro => l.lbl("TEMPO", 52.0, 18.0, !hide(Edit::Tempo)),
        Mode::Loop => {
            l.lbl("LOOP", 56.0, 18.0, true);
            arrow(l);
        }
        Mode::Tune => l.lbl("NOTE", 56.0, 32.0, true),
        Mode::Coach => {}
    }

    // big digits
    if mode == Mode::Tune {
        let nn = NOTE_NAMES[(d.note % 12) as usize];
        let oct = (d.note / 12) as i32 - 1;
        l.digit(BIG_XS[0], BIG_Y, BIG, nn.chars().next().unwrap(), true);
        l.digit(BIG_XS[1], BIG_Y, BIG, char::from_digit(oct as u32, 10).unwrap_or(' '), true);
        l.digit(BIG_XS[2], BIG_Y, BIG, ' ', false);
        l.label("#", 113.0, 15.0, nn.len() > 1, 10.0, "start");
    } else if let (Mode::Coach, Some(tc)) = (mode, d.time_check) {
        // 7-position timing indicator across the 3 big digits (3 = on the beat)
        let p = [(0, "f"), (0, "b"), (1, "f"), (1, "abf"), (1, "b"), (2, "f"), (2, "b")][tc as usize];
        let mut masks = [String::new(), String::new(), String::new()];
        masks[p.0] += p.1;
        masks[1] += "d";
        for (i, x) in BIG_XS.iter().enumerate() {
            l.digit_mask(*x, BIG_Y, BIG, &masks[i], true);
        }
        l.label("[GOOD]", 129.0, 53.0, tc == 3, 6.4, "middle");
    } else {
        let txt = match mode {
            Mode::Loop => d.loop_repeats.to_string(),
            Mode::Coach if d.show_coach_name => ["tch", "qct", "Gud", "Sud"][d.coach as usize].to_string(),
            _ => d.tempo.to_string(),
        };
        l.digits(&BIG_XS, BIG_Y, BIG, &txt, !hide(Edit::Tempo));
    }

    // right block: BEAT/PATTERN + STYLE, or PITCH
    if mode == Mode::Tune {
        l.label("PITCH", 191.0, 40.0, !hide(Edit::Pitch), 6.2, "end");
        l.digits(&RIGHT_XS, 22.0, SM, &d.pitch.to_string(), !hide(Edit::Pitch));
        l.label("Hz", 236.0, 18.0, true, 5.6, "start");
        if !d.tone_on {
            l.label("MUTE", 246.0, 78.0, true, 6.6, "end");
        }
    } else {
        if d.pattern_mode {
            l.label("PATTERN", 169.0, 36.0, !hide(Edit::Beat), 5.4, "start");
        } else {
            l.lbl("BEAT", 174.0, 32.0, !hide(Edit::Beat));
        }
        l.lbl("STYLE", 202.0, 16.0, true);
        let (st, vis) = if mode == Mode::Loop {
            (d.loop_count.to_string(), true)
        } else if mode == Mode::Coach && d.edit == Some(Edit::Guide) {
            l.lbl("GUIDE", 174.0, 46.0, true);
            (format!("{} {}", d.guide, d.beat1), d.blink)
        } else if d.edit == Some(Edit::Beat2) {
            (format!("{}{}", d.beat1, if d.beat2 == 0 { "-".into() } else { d.beat2.to_string() }), d.blink)
        } else if d.edit == Some(Edit::CountIn) || (d.count_in > 0 && !d.playing) {
            (format!("-{}", d.count_in), !hide(Edit::CountIn))
        } else if d.pattern_mode {
            (d.pattern.to_string(), !hide(Edit::Beat))
        } else if d.playing {
            (d.beat_pos.to_string(), true)
        } else if d.beat2 > 0 {
            (format!("{}{}", d.beat1, d.beat2), !hide(Edit::Beat))
        } else {
            (d.beat1.to_string(), !hide(Edit::Beat))
        };
        l.digits(&RIGHT_XS, 22.0, SM, &st, vis);
        let _ = write!(l.o, r#"<path fill="{}" d="M215,50 l3,-4 l3,4z"/>"#, l.ink.on);
        let count = if d.count_in > 0 && mode != Mode::Loop { "COUNT IN" } else { "COUNT" };
        l.label(count, 218.0, 58.0, true, 6.2, "middle");
        if d.muted {
            l.label("MUTE", 246.0, 78.0, true, 6.6, "end");
        }
    }

    // voice
    if mode == Mode::Metro && !d.pattern_mode {
        l.lbl("VOICE", 6.0, 66.0, !hide(Edit::Voice));
        l.digit(13.0, 70.0, SM, char::from_digit(d.voice as u32, 10).unwrap_or(' '), !hide(Edit::Voice));
    }

    // sun icon: reserved; the backlight never switches itself off, so it stays unlit
    l.sun(14.0, 106.0, false);

    if mode != Mode::Tune {
        l.pendulum(Some(d.pend));

        // bottom: divider, VOL, amp icon (never lit), rhythm levels
        let _ = write!(l.o, r#"<rect fill="{}" x="4" y="154" width="242" height="0.9"/>"#, l.ink.on);
        l.label("VOL", 230.0, 163.0, true, 6.0, "middle");
        let cone = |cx: f64| format!("M{},145 a2.6,2.6 0 1,0 -5.2,0 a2.6,2.6 0 1,0 5.2,0 Z", cx + 2.6);
        let f = l.fill(false);
        let _ = write!(l.o, r#"<path {f} fill-rule="evenodd" d="M222.5,137 h15 a1.5,1.5 0 0,1 1.5,1.5 v10 a1.5,1.5 0 0,1 -1.5,1.5 h-15 a1.5,1.5 0 0,1 -1.5,-1.5 v-10 a1.5,1.5 0 0,1 1.5,-1.5 Z M223,139 h14 v1.4 h-14 Z {} {}"/>"#, cone(226.0), cone(234.0));
        let xs = [8.0, 46.0, 84.0, 122.0, 160.0];
        let pat = d.pattern_mode;
        let bottom = if pat { 188.0 } else { 196.0 };
        if pat {
            for (i, t) in ["KICK", "SNARE", "HH", "OTHER1", "OTHER2"].iter().enumerate() {
                l.label(t, xs[i] + 10.0, 198.0, true, 6.0, "middle");
            }
        } else {
            l.label("ACCENT", 6.0, 166.0, true, 6.0, "start");
            l.note_icon(1, 54.0, 174.0);
            l.note_icon(2, 88.0, 174.0);
            l.note_icon(3, 124.0, 174.0);
            l.note_icon(4, 164.0, 174.0);
        }
        let lv = d.levels();
        for (i, x) in xs.iter().enumerate() {
            l.wedge(x + if pat { 0.0 } else { 4.0 }, bottom, lv[i]);
        }
        let v = (d.sliders[5] as f64 / 20.0).ceil() as u8;
        for i in 0..5u8 {
            let f = l.fill(i < v);
            let _ = write!(l.o, r#"<rect {f} x="221" y="{}" width="18" height="5"/>"#, f1(193.0 - i as f64 * 6.2));
        }
    }
}

// ------------------------------------------------------------------ knob

fn well_path(open: bool) -> String {
    let (cx, cy) = DIAL;
    let r = 106.0;
    if open {
        format!("M604,{} L{cx},{} A{r},{r} 0 0,0 {cx},{} L604,{}", cy - r, cy - r, cy + r, cy + r)
    } else {
        format!("M{cx},{} L604,{} L604,{} L{cx},{} A{r},{r} 0 0,1 {cx},{} Z", cy - r, cy - r, cy + r, cy + r, cy - r)
    }
}

/// The turning knob: knurl, bumps and dimple move; light, shadow and highlights stay fixed.
pub fn knob(angle: f64) -> String {
    let (cx, cy) = DIAL;
    let mut o = format!(
        r##"<circle cx="{cx}" cy="{cy}" r="87.3" fill="none" stroke="#ffb229" stroke-width="3.6" stroke-dasharray="1.6 1.4" transform="rotate({} {cx} {cy})"/>"##,
        f1(angle)
    );
    let _ = write!(o, r#"<circle cx="{cx}" cy="{cy}" r="87.3" fill="none" stroke="url(#knurlShade)" stroke-width="4"/>"#);
    let _ = write!(o, r#"<circle cx="{cx}" cy="{cy}" r="84" fill="url(#dialGrad)"/>"#);
    let _ = write!(o, r#"<circle cx="{cx}" cy="{cy}" r="84" fill="none" stroke="url(#edgeLit)" stroke-width="1.4" opacity="0.8"/>"#);
    let rad = angle * PI / 180.0;
    for i in 0..8 {
        let a = i as f64 * PI / 4.0 + 0.2 + rad;
        let _ = write!(o, r#"<circle cx="{}" cy="{}" r="2.3" fill="url(#bump)"/>"#, f1(cx + a.cos() * 60.0), f1(cy + a.sin() * 60.0));
    }
    let da = (-28.0f64).atan2(46.0) + rad;
    let dr = (46.0f64).hypot(28.0);
    let (dx, dy) = (f1(cx + da.cos() * dr), f1(cy + da.sin() * dr));
    let _ = write!(o, r#"<circle cx="{dx}" cy="{dy}" r="22" fill="url(#dimple)"/><circle cx="{dx}" cy="{dy}" r="21.3" fill="none" stroke="url(#dimpleWall)" stroke-width="1.6"/>"#);
    let _ = write!(
        o,
        r##"<ellipse cx="{}" cy="{}" rx="38" ry="22" transform="rotate(-35 {} {})" fill="#fff" opacity="0.16" filter="url(#blur4)"/>"##,
        cx - 26.0,
        cy - 34.0,
        cx - 26.0,
        cy - 34.0
    );
    doc(&o, &["knurlShade", "dialGrad", "edgeLit", "bump", "dimple", "dimpleWall", "blur4"], None)
}

// ------------------------------------------------------------------ buttons / caps / LEDs

pub fn button_cap(bd: &ButtonDef, pressed: bool) -> String {
    let (cx, cy, w, h, size) = (bd.cx, bd.cy, bd.w, bd.h, bd.size);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let parts: Vec<&str> = bd.label.split('/').collect();
    let multi = parts.len() > 1 && bd.style != Style::Single;
    let fill = if bd.style == Style::Light { "#1a1a1a" } else { "#f2f2f2" };
    let mut t = String::new();
    if multi {
        let _ = write!(t, r#"<text x="{cx}" y="{}" font-size="{}" text-anchor="middle" fill="{fill}">{}/</text>"#, cy - 1.5, size - 0.8, parts[0]);
        let _ = write!(t, r#"<text x="{cx}" y="{}" font-size="{}" text-anchor="middle" fill="{fill}">{}</text>"#, cy + 8.0, size - 0.8, parts[1]);
    } else {
        let _ = write!(t, r#"<text x="{cx}" y="{}" font-size="{size}" text-anchor="middle" fill="{fill}">{}</text>"#, cy + size * 0.36, bd.label);
    }
    let bw = (w - 10.0).min(bd.label.len() as f64 * size * 0.66 + 8.0);
    let bh = if multi { 22.0 } else { size + 6.0 };
    let deco = match bd.style {
        Style::Boxed => format!(r##"<rect x="{}" y="{}" width="{bw}" height="{bh}" rx="2" fill="none" stroke="#e8e8e8" stroke-width="1.1"/>"##, cx - bw / 2.0, cy - bh / 2.0),
        Style::Light => format!(r##"<rect x="{}" y="{}" width="{bw}" height="{bh}" rx="2" fill="#d6d8da"/>"##, cx - bw / 2.0, cy - bh / 2.0),
        _ => String::new(),
    };
    // pressed: the cap sinks into the static hole (slightly smaller and darker, shadow and sheen fade)
    let (tr, shadow, sheen) = if pressed {
        let s = 0.955;
        (format!(r#" transform="translate({},{}) scale({s})""#, cx * (1.0 - s), cy * (1.0 - s)), 0.15 * 0.75, 0.35)
    } else {
        (String::new(), 0.75, 1.0)
    };
    let mut o = format!("<g{tr}>");
    let _ = write!(o, r##"<rect x="{}" y="{}" width="{w}" height="{h}" rx="4" fill="#000" opacity="{shadow}" filter="url(#blur1)"/>"##, x + 1.2, y + 2.4);
    let _ = write!(o, r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="url(#btnGrad)" stroke="#070708" stroke-width="1"/>"##);
    let _ = write!(o, r#"<g opacity="{sheen}"><rect x="{}" y="{}" width="{}" height="{}" rx="3.4" fill="none" stroke="url(#capEdge)" stroke-width="1.4"/><rect x="{}" y="{}" width="{}" height="{}" rx="2.5" fill="url(#capTop)"/></g>"#,
        x + 0.8, y + 0.8, w - 1.6, h - 1.6, x + 3.0, y + 1.6, w - 6.0, h * 0.4);
    o += &deco;
    o += &t;
    if pressed {
        let _ = write!(o, r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="#000" opacity="0.2"/>"##);
    }
    o += "</g>";
    doc(&o, &["btnGrad", "capEdge", "capTop", "blur1"], None)
}

/// Slider cap drawn at y = 0 (moved by the compositor).
pub fn slider_cap(i: usize) -> String {
    let x = SLIDERS[i];
    let mut o = String::new();
    let _ = write!(o, r##"<rect x="{}" y="-11" width="17" height="27" rx="1.6" fill="#000" opacity="0.45" filter="url(#blur1)"/>"##, x - 7.0);
    let _ = write!(o, r##"<rect x="{}" y="-13.5" width="16" height="27" rx="1.6" fill="#c76f04"/>"##, x - 8.0);
    let _ = write!(o, r##"<rect x="{}" y="-13.5" width="16" height="14" rx="1.6" fill="url(#capGrad)"/>"##, x - 8.0);
    let _ = write!(o, r##"<rect x="{}" y="0.5" width="16" height="11" rx="1" fill="#f39a0c"/>"##, x - 8.0);
    let _ = write!(o, r##"<rect x="{}" y="0" width="16" height="1.3" fill="#ffd27a" opacity="0.9"/>"##, x - 8.0);
    let _ = write!(o, r##"<rect x="{}" y="-13.5" width="16" height="27" rx="1.6" fill="none" stroke="#9c5704" stroke-width="0.6"/>"##, x - 8.0);
    // drawn around y=40 so the blur filter region stays inside the viewBox
    doc(&format!(r#"<g transform="translate(0,40)">{o}</g>"#), &["capGrad", "blur1"], None)
}
pub const CAP_REF_Y: f64 = 40.0;

pub fn led_lit(i: usize) -> String {
    let (x, y) = LEDS[i];
    let c = if i == 0 { "#ff3b26" } else { "#46ff6a" };
    let o = format!(
        r#"<rect x="{}" y="{}" width="28" height="16" rx="6" fill="{c}" opacity="0.55" filter="url(#glow)"/><rect x="{x}" y="{y}" width="20" height="8" rx="2" fill="{c}"/><rect x="{}" y="{}" width="16" height="2" rx="1" fill="white" opacity="0.45"/>"#,
        x - 4.0,
        y - 4.0,
        x + 2.0,
        y + 1.0
    );
    doc(&o, &["glow"], None)
}

pub fn notice(text: &str) -> String {
    let w = 30.0 + text.len() as f64 * 6.6;
    let x = W / 2.0 - w / 2.0;
    let o = format!(
        r##"<rect x="{x}" y="14" width="{w}" height="30" rx="8" fill="#141517" opacity="0.94"/><text x="{}" y="33.5" font-size="12" font-weight="normal" fill="#e6e7e8" text-anchor="middle">{text}</text>"##,
        W / 2.0
    );
    doc(&o, &[], None)
}

// ------------------------------------------------------------------ static background

fn body_paint() -> &'static str {
    r#"<rect x="6" y="6" width="598" height="784" rx="24" fill="url(#body)"/><rect x="6" y="6" width="598" height="784" rx="24" fill="url(#paint)"/><rect x="6" y="6" width="598" height="784" rx="24" fill="url(#sheen)"/>"#
}

fn milled(o: &mut String, x: f64, y: f64, w: f64, h: f64, r: f64) {
    for (dx, stroke, sw, op) in [(-0.8, "#000", 1.5, 0.55), (0.9, "#fff", 1.2, 0.3), (0.0, "#151618", 0.8, 0.9)] {
        let _ = write!(o, r#"<rect x="{}" y="{}" width="{w}" height="{h}" rx="{r}" fill="none" stroke="{stroke}" stroke-width="{sw}" opacity="{op}"/>"#, x + dx, y + dx);
    }
}

fn button_socket(o: &mut String, bd: &ButtonDef) {
    let (x, y, w, h) = (bd.cx - bd.w / 2.0, bd.cy - bd.h / 2.0, bd.w, bd.h);
    if bd.rim {
        // wider chamfer around POWER / WRITE so they can be found blind
        let _ = write!(o, r##"<rect x="{}" y="{}" width="{}" height="{}" rx="8.5" fill="url(#chamferBig)"/><rect x="{}" y="{}" width="{}" height="{}" rx="8.5" fill="none" stroke="#2c2e30" stroke-width="0.7" opacity="0.6"/>"##,
            x - 6.5, y - 6.5, w + 13.0, h + 13.0, x - 6.5, y - 6.5, w + 13.0, h + 13.0);
    }
    let _ = write!(o, r#"<rect x="{}" y="{}" width="{}" height="{}" rx="6" fill="url(#hole)" stroke="url(#edgeHole)" stroke-width="1.2"/>"#, x - 3.0, y - 3.0, w + 6.0, h + 6.0);
}

pub fn background(paint_uri: &str) -> String {
    let mut o = String::new();
    o += r##"<rect x="8" y="10" width="594" height="782" rx="24" fill="#000" opacity="0.55" filter="url(#caseBlur3)"/>"##;
    o += r#"<clipPath id="bodyClip"><rect x="6" y="6" width="598" height="784" rx="24"/></clipPath>"#;
    o += body_paint();

    // black top panel with branding and LEDs (logo is a TODO; v1 text branding)
    o += r#"<path d="M36,470 V43 a5,5 0 0,1 5,-5 H329 a5,5 0 0,1 5,5 V470 Z" fill="url(#black)"/>"#;
    o += r##"<g transform="translate(185,94)"><rect x="-112" y="-20" width="20" height="22" rx="2" fill="none" stroke="#c4c6c8" stroke-width="2.4"/><path d="M-106,-14 v12 h9 v-6 h-9" fill="none" stroke="#c4c6c8" stroke-width="2"/><text x="12" y="1" font-size="32" font-weight="900" text-anchor="middle" fill="#c4c6c8" letter-spacing="1" transform="scale(1.08,1)">KADENCE</text></g>"##;
    o += r##"<text x="185" y="136" font-size="11.5" text-anchor="middle" fill="#c4c6c8" letter-spacing="0.6">METRONOME <tspan font-style="italic" font-weight="normal">Dr.Tick</tspan></text>"##;
    o += r##"<text x="185" y="180" font-size="42" text-anchor="middle" fill="#c9cbcd" letter-spacing="2" transform="translate(185,0) scale(1.08,1) translate(-185,0)">KD-90</text>"##;
    for (i, (x, y)) in LEDS.iter().enumerate() {
        let c = if i == 0 { "#3a2a2a" } else { "#2a3a2c" };
        let _ = write!(o, r#"<rect x="{x}" y="{y}" width="20" height="8" rx="2" fill="{c}"/>"#);
    }
    o += r##"<rect x="56" y="214" width="258" height="208" rx="3" fill="#050505" stroke="#2a2a2a"/>"##;

    // speaker grille
    for i in 0..13 {
        let y = 52 + i * 14;
        let _ = write!(o, r##"<rect x="346" y="{y}" width="234" height="7" rx="3.5" fill="#1d1e20"/><rect x="348" y="{}" width="230" height="1.2" fill="#fff" opacity="0.18"/>"##, y + 7);
    }

    for bd in &BUTTONS {
        button_socket(&mut o, bd);
    }

    let _ = write!(o, r#"<g font-size="{PRINT_SIZE}" fill="{PRINT}"><text x="352" y="425">TEMPO/</text><text x="352" y="436">VALUE/</text><rect x="351" y="440" width="30" height="11" rx="1" fill="none" stroke="{PRINT}" stroke-width="1"/><text x="354.5" y="448.8">NOTE</text></g>"#);

    // knob well (a lower step of the case) and the fixed parts of the knob
    let (cx, cy) = DIAL;
    let (well, wall) = (well_path(false), well_path(true));
    let _ = write!(o, r##"<clipPath id="wellClip"><path d="{well}"/></clipPath><g clip-path="url(#wellClip)">{}<path d="{well}" fill="#000" opacity="0.06"/>"##, body_paint());
    let _ = write!(o, r##"<path d="{wall}" fill="none" stroke="#000" stroke-width="10" opacity="0.4" filter="url(#blur4)" transform="translate(3,4)"/>"##);
    let _ = write!(o, r##"<path d="M{},{} A106,106 0 0,0 {},{}" fill="none" stroke="#d5780c" stroke-width="7" opacity="0.4" filter="url(#blur2)"/>"##, cx - 40.0, cy - 98.0, cx - 40.0, cy + 98.0);
    let _ = write!(o, r##"<circle cx="{}" cy="{}" r="89" fill="#000" opacity="0.55" filter="url(#blur4)"/></g>"##, cx + 7.0, cy + 9.0);
    let _ = write!(o, r#"<path d="{wall}" fill="none" stroke="url(#edgeHole)" stroke-width="1.6" clip-path="url(#bodyClip)"/>"#);
    let _ = write!(o, r##"<circle cx="{}" cy="{}" r="89.5" fill="#a85a00"/><circle cx="{cx}" cy="{cy}" r="89" fill="#d27a03"/>"##, cx + 1.4, cy + 2.4);

    // mic
    let _ = write!(o, r#"<text x="368" y="584" font-size="{PRINT_SIZE}" fill="{PRINT}" text-anchor="middle">MIC</text>"#);
    for i in 0..3 {
        let _ = write!(o, r##"<rect x="{}" y="590" width="3.6" height="22" rx="1.8" fill="#1d1e20"/>"##, 358 + i * 8);
    }

    // milled outline around screen panel + sliders, shared grooves for screen / sliders / VOL
    milled(&mut o, 36.0, 38.0, 298.0, 588.0, 5.0);
    o += r##"<rect x="36" y="471.9" width="298" height="1.2" fill="#fff" opacity="0.3"/><rect x="36" y="470.6" width="298" height="0.8" fill="#151618" opacity="0.9"/>"##;
    o += r##"<rect x="265.2" y="471" width="1.5" height="155" fill="#000" opacity="0.55"/><rect x="266.9" y="471" width="1.2" height="155" fill="#fff" opacity="0.3"/><rect x="265.6" y="471" width="0.8" height="155" fill="#151618" opacity="0.9"/>"##;
    let _ = write!(o, r#"<text x="300" y="489" font-size="{PRINT_SIZE}" fill="{PRINT}" text-anchor="middle">VOL</text>"#);
    for x in SLIDERS {
        let _ = write!(o, r##"<rect x="{}" y="{SL_TOP}" width="13" height="{}" fill="#070708"/><rect x="{}" y="{SL_TOP}" width="13" height="{}" fill="none" stroke="url(#edgeHole)" stroke-width="1"/>"##,
            x - 6.5, SL_BOT - SL_TOP, x - 6.5, SL_BOT - SL_TOP);
        for k in 0..=10 {
            let y = SL_BOT - 14.0 - (k as f64 / 10.0) * (SL_BOT - SL_TOP - 28.0);
            let _ = write!(o, r#"<rect x="{}" y="{}" width="{}" height="1.1" fill="{PRINT}"/>"#, x + 10.0, f1(y - 0.55), if k % 5 == 0 { 7.0 } else { 4.5 });
        }
    }

    let _ = write!(o, r#"<path d="M46,746 v6 h24 M154,752 h24 v-6" fill="none" stroke="{PRINT}" stroke-width="1"/><text x="112" y="755.5" font-size="{PRINT_SIZE}" fill="{PRINT}" text-anchor="middle">COUNT IN</text>"#);

    // rounded case edge: full rounding on the upper level, a small edge along the lowered knob well
    let _ = write!(o, r##"<mask id="upperLevel"><rect x="0" y="0" width="610" height="800" fill="#fff"/><path d="{well}" fill="#000"/></mask>"##);
    o += r#"<g clip-path="url(#wellClip)"><g clip-path="url(#bodyClip)"><rect x="7.2" y="7.2" width="595.6" height="781.6" rx="22.8" fill="none" stroke="url(#bevelH)" stroke-width="2.6" filter="url(#caseBlur1)"/></g></g>"#;
    o += r##"<g clip-path="url(#bodyClip)" mask="url(#upperLevel)"><rect x="9.5" y="9.5" width="591" height="777" rx="21" fill="none" stroke="url(#bevelV)" stroke-width="8" filter="url(#caseBlur2)"/><rect x="9.5" y="9.5" width="591" height="777" rx="21" fill="none" stroke="url(#bevelH)" stroke-width="8" filter="url(#caseBlur2)"/><rect x="7" y="7" width="596" height="782" rx="23" fill="none" stroke="#fff" stroke-width="1" opacity="0.12"/></g>"##;
    o += r##"<rect x="6" y="6" width="598" height="784" rx="24" fill="none" stroke="#2e3032" stroke-width="1.2"/>"##;

    let ids: Vec<&str> = DEFS.iter().map(|(id, _)| *id).collect();
    doc(&o, &ids, Some(paint_uri))
}
