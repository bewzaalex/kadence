//! SVG sources for every part of the panel (port of docs/design/mockup/panel.js).
//!
//! The case is split into a static background and small dynamic layers (knob, button caps,
//! slider caps, LEDs; the LCD is in `lcd.rs`) so only what changes is re-rendered.

use std::f64::consts::PI;
use std::fmt::Write;

use crate::device::Btn;

pub const W: f64 = 610.0;
pub const H: f64 = 800.0;
pub const DIAL: (f64, f64) = (492.0, 524.0);
/// The dial's radius, to the outside of its knurled ring.
const DIAL_R: f64 = 89.0;
pub const SLIDERS: [f64; 6] = [70.0, 112.0, 154.0, 196.0, 238.0, 300.0];
pub const SL_TOP: f64 = 494.0;
pub const SL_BOT: f64 = 610.0;
/// How far each end of a slider's slot is drawn inside SL_TOP..SL_BOT.
const SLOT_INSET: f64 = 1.5;
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
    /// a small key of the same black glass as the screen panel, with a chamfered rim
    Glass,
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
    /// its key, shown on a badge (K); empty: none (a function not released yet has no key)
    pub key: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn b(id: Btn, cx: f64, cy: f64, w: f64, h: f64, label: &'static str, style: Style, size: f64, rim: bool, key: &'static str) -> ButtonDef {
    ButtonDef { id, cx, cy, w, h, label, style, size, rim, key }
}

pub const BUTTONS: [ButtonDef; 17] = [
    b(Btn::Power, 384.0, 322.0, 60.0, 34.0, "POWER", Style::Boxed, 9.6, true, "Ctrl+Q"),
    b(Btn::Light, 458.0, 322.0, 60.0, 34.0, "LIGHT", Style::Plain, 9.6, false, "L"),
    b(Btn::Mute, 532.0, 322.0, 60.0, 34.0, "MUTE", Style::Plain, 9.6, false, "M"),
    b(Btn::Write, 384.0, 378.0, 60.0, 34.0, "WRITE", Style::Plain, 9.6, true, "W"),
    b(Btn::Memory, 458.0, 378.0, 60.0, 34.0, "MEMORY", Style::Plain, 8.8, false, "R"),
    b(Btn::Midi, 532.0, 378.0, 60.0, 34.0, "MIDI", Style::Plain, 9.6, false, ""),
    b(Btn::Tempo, 75.0, 670.0, 60.0, 36.0, "TEMPO", Style::Plain, 9.6, false, "Esc"),
    b(Btn::Loop, 150.0, 670.0, 60.0, 36.0, "LOOP", Style::Plain, 9.6, false, ""),
    b(Btn::Tune, 225.0, 670.0, 60.0, 36.0, "TUNE", Style::Boxed, 9.6, false, "A"),
    b(Btn::Coach, 300.0, 670.0, 60.0, 36.0, "COACH", Style::Light, 9.6, false, ""),
    b(Btn::Beat, 75.0, 724.0, 60.0, 36.0, "BEAT/PATTERN", Style::Plain, 9.4, false, "B"),
    b(Btn::Voice, 150.0, 724.0, 60.0, 36.0, "VOICE", Style::Plain, 9.6, false, "V"),
    b(Btn::Pitch, 225.0, 724.0, 60.0, 36.0, "PITCH", Style::Boxed, 9.6, false, "P"),
    b(Btn::Trig, 300.0, 724.0, 60.0, 36.0, "TRIG/GUIDE", Style::Light, 9.4, false, ""),
    b(Btn::Tap, 380.0, 697.0, 54.0, 90.0, "TAP", Style::Plain, 12.0, false, "T"),
    b(Btn::Start, 494.0, 697.0, 132.0, 90.0, "START/STOP", Style::Single, 13.0, false, "Space"),
    // under the screen, in line with the VOL slider
    b(Btn::Help, 300.0, 446.0, 26.0, 20.0, "?", Style::Glass, 12.5, false, "F1"),
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
    ("glassChamfer", r##"<linearGradient id="glassChamfer" x1="0" y1="0" x2="0.35" y2="1"><stop offset="0" stop-color="#76797c"/><stop offset="0.45" stop-color="#232426"/><stop offset="1" stop-color="#000"/></linearGradient>"##),
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

pub(super) fn f1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub(super) fn pts(p: &[(f64, f64)]) -> String {
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
    if bd.style == Style::Glass {
        return glass_cap(bd, pressed);
    }
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

/// A black glass key: a chamfered rim lit from the top left, the face with the screen's sheen,
/// and its label printed like the case's lettering.
fn glass_cap(bd: &ButtonDef, pressed: bool) -> String {
    let (cx, cy, w, h) = (bd.cx, bd.cy, bd.w, bd.h);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let tr = if pressed {
        let s = 0.955;
        format!(r#" transform="translate({},{}) scale({s})""#, cx * (1.0 - s), cy * (1.0 - s))
    } else {
        String::new()
    };
    let mut o = format!("<g{tr}>");
    let _ = write!(o, r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="3.5" fill="url(#glassChamfer)"/>"##);
    let _ = write!(o, r##"<rect x="{}" y="{}" width="{}" height="{}" rx="2.2" fill="url(#black)"/>"##, x + 1.6, y + 1.6, w - 3.2, h - 3.2);
    let _ = write!(o, r##"<rect x="{}" y="{}" width="{}" height="{}" rx="2.2" fill="url(#glass)"/>"##, x + 1.6, y + 1.6, w - 3.2, h - 3.2);
    let _ = write!(o, r#"<text x="{cx}" y="{}" font-size="{}" text-anchor="middle" fill="{PRINT}">{}</text>"#, cy + bd.size * 0.36, bd.size, bd.label);
    if pressed {
        let _ = write!(o, r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="3.5" fill="#000" opacity="0.3"/>"##);
    }
    o += "</g>";
    doc(&o, &["glassChamfer", "black", "glass"], None)
}

/// Slider cap drawn at y = 0 (moved by the compositor).
pub fn slider_cap(i: usize) -> String {
    let x = SLIDERS[i];
    let mut o = String::new();
    let _ = write!(o, r##"<rect x="{}" y="-11" width="17" height="27" rx="1.6" fill="#000" opacity="0.45" filter="url(#blur1)"/>"##, x - 7.0);
    let _ = write!(o, r##"<rect x="{}" y="-13.5" width="16" height="27" rx="1.6" fill="#c76f04"/>"##, x - 8.0);
    // the line across the cap is centred on y = 0, like the scale's ticks
    let _ = write!(o, r##"<rect x="{}" y="-13.5" width="16" height="13.35" rx="1.6" fill="url(#capGrad)"/>"##, x - 8.0);
    let _ = write!(o, r##"<rect x="{}" y="-0.15" width="16" height="11.65" rx="1" fill="#f39a0c"/>"##, x - 8.0);
    let _ = write!(o, r##"<rect x="{}" y="-0.65" width="16" height="1.3" fill="#ffd27a" opacity="0.9"/>"##, x - 8.0);
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

/// The keyboard shortcuts overlay's area: (x, y, w, h).
pub const HELP_RECT: (f64, f64, f64, f64) = (95.0, 128.0, 420.0, 505.0);

/// Keyboard shortcuts: (keys, what they do).
const SHORTCUTS: &[(&str, &str)] = &[
    ("Space / Enter", "START/STOP"),
    ("T", "TAP"),
    ("Up / Down, + / −, wheel", "Turn the dial one click"),
    ("Right / Left", "Turn the dial ten clicks"),
    ("M", "MUTE"),
    ("Esc", "Back: close, else the TEMPO screen"),
    ("A / P", "TUNE / PITCH"),
    ("B / V", "BEAT/PATTERN / VOICE"),
    ("B + V", "Count-in"),
    ("R / W", "MEMORY / WRITE"),
    ("L", "LIGHT"),
    ("K", "Show the keys on the buttons"),
    ("F1 / ?", "This list"),
    ("Ctrl+Q", "POWER: switch off and quit"),
    ("Hold R, click POWER", "Restore factory settings"),
    ("Ctrl + / − / 0", "Window size"),
];

/// The keyboard shortcuts and the version, over the panel, in the notice's style.
pub fn help() -> String {
    let (x, y, w, h) = HELP_RECT;
    let mid = x + 170.0;
    let mut o = format!(r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="10" fill="#141517" opacity="0.96"/>"##);
    let _ = write!(o, r##"<text x="{}" y="{}" font-size="15" fill="#e6e7e8" text-anchor="middle">Keyboard shortcuts</text>"##, x + w / 2.0, y + 32.0);
    let _ = write!(
        o,
        r##"<text x="{}" y="{}" font-size="10" font-weight="normal" fill="#8d9195" text-anchor="middle">Kadence {}</text>"##,
        x + w / 2.0,
        y + 50.0,
        env!("CARGO_PKG_VERSION")
    );
    o += r#"<g font-size="12" font-weight="normal">"#;
    for (i, (keys, what)) in SHORTCUTS.iter().enumerate() {
        let ty = y + 82.0 + i as f64 * 25.0;
        let _ = write!(o, r##"<text x="{}" y="{ty}" fill="#ffa514" text-anchor="end">{keys}</text><text x="{}" y="{ty}" fill="#e6e7e8">{what}</text>"##, mid - 8.0, mid + 8.0);
    }
    let _ = write!(o, r##"</g><text x="{}" y="{}" font-size="10" font-weight="normal" fill="#8d9195" text-anchor="middle">Click anywhere or press Esc to close</text>"##, x + w / 2.0, y + h - 16.0);
    doc(&o, &[], None)
}

/// Key badges: height, text size, and the width of one character and of the padding around them.
const BADGE_H: f64 = 18.0;
const BADGE_TEXT: f64 = 12.0;
const BADGE_CHAR: f64 = 7.0;
const BADGE_PAD: f64 = 10.0;
/// How far a badge reaches over the corner it sits on, to the right and down.
const BADGE_OVER: f64 = 4.0;

fn badge_width(key: &str) -> f64 {
    BADGE_PAD + key.chars().count() as f64 * BADGE_CHAR
}

/// A key badge on the panel, its bottom-right corner at (right, bottom).
fn badge(o: &mut String, right: f64, bottom: f64, key: &str) {
    let (w, top) = (badge_width(key), bottom - BADGE_H);
    let _ = write!(
        o,
        r##"<rect x="{}" y="{top}" width="{w}" height="{BADGE_H}" rx="4" fill="#141517" opacity="0.94"/><text x="{}" y="{}" font-size="{BADGE_TEXT}" fill="#ffa514" text-anchor="middle">{key}</text>"##,
        right - w,
        right - w / 2.0,
        bottom - 5.0
    );
}

/// A key badge sitting on the top-right corner (x, y) of what it labels.
fn corner_badge(o: &mut String, x: f64, y: f64, key: &str) {
    badge(o, x + BADGE_OVER, y + BADGE_OVER, key);
}

/// Every button's key on a badge on its top-right corner, plus the dial's (K).
pub fn badges() -> String {
    let mut o = String::new();
    for bd in BUTTONS.iter().filter(|bd| !bd.key.is_empty()) {
        corner_badge(&mut o, bd.cx + bd.w / 2.0, bd.cy - bd.h / 2.0, bd.key);
    }
    // the dial's badge is centred on its top edge
    let dial = "+ / −";
    badge(&mut o, DIAL.0 + badge_width(dial) / 2.0, DIAL.1 - DIAL_R + BADGE_OVER, dial);
    doc(&o, &[], None)
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
    if bd.style == Style::Glass {
        // a thin dark gap in the glass panel, like the screen's bezel
        let _ = write!(o, r##"<rect x="{}" y="{}" width="{}" height="{}" rx="4.5" fill="#020202" stroke="#2a2a2a" stroke-width="0.8"/>"##, x - 1.5, y - 1.5, w + 3.0, h + 3.0);
        return;
    }
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

    // black top panel with the lettering (drawn in docs/design/logo/, exported to assets/) and LEDs
    o += r#"<path d="M36,470 V43 a5,5 0 0,1 5,-5 H329 a5,5 0 0,1 5,5 V470 Z" fill="url(#black)"/>"#;
    o += include_str!("../../assets/lettering.svg");
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
        // the slot ends just inside the cap's travel, so a cap at either end covers it
        let (top, h) = (SL_TOP + SLOT_INSET, SL_BOT - SL_TOP - 2.0 * SLOT_INSET);
        let _ = write!(o, r##"<rect x="{}" y="{top}" width="13" height="{h}" fill="#070708"/><rect x="{}" y="{top}" width="13" height="{h}" fill="none" stroke="url(#edgeHole)" stroke-width="1"/>"##,
            x - 6.5, x - 6.5);
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
