//! The LCD: every segment, label and icon of the display, lit or unlit for the device state.
//!
//! The layout is drawn in its own coordinates (842 × 702 units, the display glass) and scaled
//! into the panel's LCD rectangle, so positions here are the display's own measurements.
//! Unlit segments are drawn faintly, as on a real LCD.

use std::fmt::Write;

use super::panel::{doc, f1, pts, LCD_RECT};
use crate::device::{Device, Edit, Mode, PEND_N};

/// The display glass in layout units.
const GLASS: (f64, f64) = (842.0, 702.0);
/// Layout origin: everything below is placed relative to this corner.
const ORIGIN: (f64, f64) = (29.0, 28.0);

/// A seven-segment digit cell: width, height, segment thickness.
#[derive(Clone, Copy)]
struct Cell {
    w: f64,
    h: f64,
    t: f64,
}
const BIG: Cell = Cell { w: 68.0, h: 146.0, t: 18.0 };
/// The memory and voice digits.
const MID: Cell = Cell { w: 41.0, h: 88.0, t: 11.5 };
/// The STYLE digits.
const SMALL: Cell = Cell { w: 35.0, h: 87.0, t: 10.5 };
/// Italic slant of all digits, in degrees.
const SLANT: f64 = 10.0;

const BIG_XS: [f64; 3] = [355.0, 429.0, 502.0];
const BIG_Y: f64 = 51.0;
const MEM_XS: [f64; 2] = [136.5, 181.5];
const MEM_Y: f64 = 80.0;
const VOICE_XY: (f64, f64) = (75.5, 235.0);
/// STYLE block: the count digit, two style digits, and the second bar after the plus sign.
const STYLE_XS: [f64; 4] = [679.0, 720.0, 759.0, 813.0];
const STYLE_Y: f64 = 110.0;
const PLUS: (f64, f64) = (795.5, 153.5);

/// Left half of the pendulum fan, outermost bar first, then the centre bar:
/// top-left, top-right, bottom-right, bottom-left. The right half mirrors it.
const FAN_LEFT: [[(f64, f64); 4]; 14] = [
    [(62.0, 354.0), (84.0, 341.0), (205.0, 549.0), (184.0, 549.0)],
    [(88.0, 338.0), (111.0, 327.0), (228.0, 549.0), (208.0, 549.0)],
    [(115.0, 324.0), (138.0, 313.0), (250.0, 549.0), (232.0, 549.0)],
    [(143.0, 313.0), (166.0, 301.0), (270.0, 549.0), (254.0, 549.0)],
    [(170.0, 299.0), (194.0, 289.0), (292.0, 549.0), (273.0, 549.0)],
    [(199.0, 289.0), (223.0, 280.0), (311.0, 549.0), (295.0, 549.0)],
    [(227.0, 277.0), (252.0, 271.0), (331.0, 549.0), (314.0, 549.0)],
    [(257.0, 272.0), (281.0, 262.0), (349.0, 549.0), (334.0, 549.0)],
    [(286.0, 262.0), (311.0, 256.0), (368.0, 549.0), (352.0, 549.0)],
    [(316.0, 258.0), (341.0, 251.0), (386.0, 549.0), (371.0, 549.0)],
    [(346.0, 253.0), (371.0, 247.0), (404.0, 549.0), (389.0, 549.0)],
    [(376.0, 249.0), (401.0, 244.0), (421.0, 549.0), (407.0, 549.0)],
    [(407.0, 243.0), (432.0, 243.0), (439.0, 549.0), (424.0, 549.0)],
    [(436.5, 242.0), (462.5, 242.0), (457.0, 549.0), (442.0, 549.0)],
];
/// How far the peak of each fan bar's top rises above its corners.
const FAN_PEAK: f64 = 9.0;
/// The fan is symmetric around this x.
const FAN_AXIS: f64 = 449.5;

/// Level wedges: left edges; each is five bars rising to the right.
const WEDGE_XS: [f64; 5] = [54.0, 192.0, 331.0, 469.0, 607.0];
const WEDGE_BOTTOM: f64 = 672.0;
/// Master volume bars, bottom to top: top edge of each bar.
const VOL_BARS: [f64; 5] = [686.0, 662.0, 639.0, 615.0, 592.0];

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

const NOTE_NAMES: [&str; 12] = ["C", "C#", "d", "d#", "E", "F", "F#", "G", "G#", "A", "A#", "b"];

struct Ink {
    on: &'static str,
    /// opacity of unlit segments
    gh: f64,
    /// the background colour, for lettering cut out of a lit box
    inv: &'static str,
}

struct Lcd {
    o: String,
    ink: Ink,
    /// false while the display is off: nothing lights
    power: bool,
}

impl Lcd {
    fn fill(&self, lit: bool) -> String {
        if lit && self.power {
            format!(r#"fill="{}""#, self.ink.on)
        } else {
            format!(r#"fill="{}" opacity="{}""#, self.ink.on, self.ink.gh)
        }
    }

    /// A printed label: `x` left edge, `base` baseline, `cap` capital height, `w` its width
    /// (all of the ink, so the font's side bearings are added around it).
    fn text(&mut self, txt: &str, x: f64, base: f64, cap: f64, w: f64, lit: bool) {
        let f = self.fill(lit);
        let _ = write!(
            self.o,
            r#"<text {f} x="{}" y="{base}" font-size="{}" textLength="{}" lengthAdjust="spacingAndGlyphs">{txt}</text>"#,
            x - 2.5,
            f1(cap / 0.714),
            w + 5.0
        );
    }

    fn poly(&mut self, p: &[(f64, f64)], lit: bool) {
        let f = self.fill(lit);
        let _ = write!(self.o, r#"<polygon {f} points="{}"/>"#, pts(p));
    }

    /// Seven-segment digit with mitred segment ends, slanted; (x, y) is the top-left corner.
    fn digit_mask(&mut self, x: f64, y: f64, c: Cell, mask: &str, on: bool) {
        let Cell { w, h, t } = c;
        let (g, m, s) = (t * 0.12, h / 2.0, t / 2.0);
        let segs: [(char, Vec<(f64, f64)>); 7] = [
            ('a', vec![(g, 0.0), (w - g, 0.0), (w - t - g, t), (t + g, t)]),
            ('b', vec![(w, g), (w, m - s - g), (w - s, m - g), (w - t, m - s - g), (w - t, t + g)]),
            ('c', vec![(w - s, m + g), (w, m + s + g), (w, h - g), (w - t, h - t - g), (w - t, m + s + g)]),
            ('d', vec![(t + g, h - t), (w - t - g, h - t), (w - g, h), (g, h)]),
            ('e', vec![(s, m + g), (t, m + s + g), (t, h - t - g), (0.0, h - g), (0.0, m + s + g)]),
            ('f', vec![(0.0, g), (t, t + g), (t, m - s - g), (s, m - g), (0.0, m - s - g)]),
            ('g', vec![(s + g, m), (t + g, m - s), (w - t - g, m - s), (w - s - g, m), (w - t - g, m + s), (t + g, m + s)]),
        ];
        let _ = write!(self.o, r#"<g transform="translate({x},{y}) skewX(-{SLANT})">"#);
        for (k, p) in &segs {
            self.poly(p, on && mask.contains(*k));
        }
        self.o += "</g>";
    }

    fn digit(&mut self, x: f64, y: f64, c: Cell, ch: char, on: bool) {
        self.digit_mask(x, y, c, glyph(ch), on);
    }

    /// Right-aligned text into the given digit cells.
    fn digits(&mut self, xs: &[f64], y: f64, c: Cell, text: &str, on: bool) {
        let chars: Vec<char> = text.chars().collect();
        for (i, x) in xs.iter().enumerate() {
            let k = (i + chars.len()) as isize - xs.len() as isize;
            let ch = if k >= 0 { chars[k as usize] } else { ' ' };
            self.digit(*x, y, c, ch, on);
        }
    }

    /// The pendulum fan; `lit` is the lit bar (0 = far left), or none.
    fn fan(&mut self, lit: Option<usize>) {
        let n = FAN_LEFT.len();
        for i in 0..PEND_N {
            let quad: Vec<(f64, f64)> = if i < n {
                FAN_LEFT[i].to_vec()
            } else {
                // mirrored: the left bar's top-right corner becomes this bar's top-left, and so on
                let q = FAN_LEFT[PEND_N - 1 - i];
                [q[1], q[0], q[3], q[2]].iter().map(|&(x, y)| (2.0 * FAN_AXIS - x, y)).collect()
            };
            // the top edge rises to a peak in its middle, along the bar's direction
            let (top, bottom) = (((quad[0].0 + quad[1].0) / 2.0, (quad[0].1 + quad[1].1) / 2.0), ((quad[2].0 + quad[3].0) / 2.0, (quad[2].1 + quad[3].1) / 2.0));
            let (dx, dy) = (top.0 - bottom.0, top.1 - bottom.1);
            let len = (dx * dx + dy * dy).sqrt();
            let peak = (top.0 + dx / len * FAN_PEAK, top.1 + dy / len * FAN_PEAK);
            self.poly(&[quad[0], peak, quad[1], quad[2], quad[3]], lit == Some(i));
        }
    }

    /// A level wedge: five bars under one rising slope; `level` bars from the left are lit.
    fn wedge(&mut self, x0: f64, level: u8) {
        let top = |x: f64| WEDGE_BOTTOM - (x - x0) * 0.55;
        for i in 0..5 {
            let (a, b) = (x0 + i as f64 * 21.0, x0 + i as f64 * 21.0 + 16.0);
            self.poly(&[(a, top(a)), (b, top(b)), (b, WEDGE_BOTTOM), (a, WEDGE_BOTTOM)], (i as u8) < level);
        }
    }

    fn note_head(&mut self, cx: f64, cy: f64, lit: bool) {
        let f = self.fill(lit);
        let _ = write!(self.o, r#"<ellipse {f} cx="{cx}" cy="{cy}" rx="8.2" ry="6" transform="rotate(-25 {cx} {cy})"/>"#);
    }

    /// A note: head centre (cx, cy), stem up on its right, with `flags` flags.
    fn note(&mut self, cx: f64, cy: f64, flags: u8, lit: bool) {
        self.note_head(cx, cy, lit);
        let f = self.fill(lit);
        let sx = cx + 5.2;
        let _ = write!(self.o, r#"<rect {f} x="{sx}" y="{}" width="3" height="{}"/>"#, cy - 36.0, 36.0);
        for k in 0..flags {
            let y = cy - 36.0 + k as f64 * 10.0;
            let _ = write!(self.o, r#"<path {f} d="M{},{y} c4,5 11,9 9,20 c-1,-7 -5,-11 -9,-12 Z"/>"#, sx + 2.5);
        }
    }

    /// An eighth rest with its top-left near (x, y).
    fn rest(&mut self, x: f64, y: f64, lit: bool) {
        let f = self.fill(lit);
        let _ = write!(
            self.o,
            r#"<path {f} d="M{},{} a4,4 0 1,1 4.5,3 c3,0 5,-1.5 7,-4 l-8,28 h-3 l7,-22 c-3,2 -6,2.5 -9,1.5 Z"/>"#,
            x + 1.0,
            y + 4.0
        );
    }

    fn icons_rhythm(&mut self, lit: bool) {
        // quarter, eighth, two sixteenths, triplet
        self.note(247.0, 620.0, 0, lit);
        self.rest(366.0, 588.0, lit);
        self.note(391.0, 620.0, 1, lit);
        self.rest(469.0, 588.0, lit);
        self.note(489.0, 620.0, 2, lit);
        self.rest(517.0, 588.0, lit);
        self.note(537.0, 620.0, 2, lit);
        for cx in [632.0, 654.0, 676.0] {
            self.note(cx, 620.0, 0, lit);
        }
        let f = self.fill(lit);
        let _ = write!(self.o, r#"<rect {f} x="637" y="584" width="50" height="6"/>"#);
        self.text("3", 657.0, 581.0, 19.0, 11.0, lit);
    }

    /// The light bulb (never lit: the backlight never switches itself off).
    fn sun(&mut self) {
        let f = self.fill(false);
        let (cx, cy) = (102.5, 511.0);
        let _ = write!(self.o, r#"<circle cx="{cx}" cy="{cy}" r="10" fill="none" stroke="{}" stroke-width="4" opacity="{}"/>"#, self.ink.on, self.ink.gh);
        let _ = write!(self.o, r#"<rect {f} x="96" y="524" width="13" height="4"/><rect {f} x="96" y="530" width="13" height="4"/><rect {f} x="98.5" y="536" width="8" height="5"/>"#);
        for a in [0.0, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0] {
            let _ = write!(self.o, r#"<rect {f} x="{}" y="{}" width="4" height="7" transform="rotate({a} {cx} {cy})"/>"#, cx - 2.0, cy - 23.0);
        }
    }

    /// The amp icon (never lit: there is no amp simulator).
    fn amp(&mut self) {
        let f = self.fill(false);
        let bg = self.ink.inv;
        let _ = write!(
            self.o,
            r#"<g {f}><path d="M770,484 v-4 a4,4 0 0,1 4,-4 h40 a4,4 0 0,1 4,4 v4 h-5 v-3 h-38 v3 Z"/><rect x="758" y="484" width="71" height="58" rx="5"/></g><g fill="{bg}"><circle cx="776" cy="494" r="2.6"/><circle cx="793.5" cy="494" r="2.6"/><circle cx="811" cy="494" r="2.6"/><circle cx="779" cy="521" r="12"/><circle cx="808" cy="521" r="12"/></g>"#
        );
    }
}

/// The LCD's ink for the current light: lit segments, unlit segments, and the background.
fn ink(d: &Device) -> Ink {
    // powering off turns the light and the segments off at once
    let light = if d.backlight && d.power { d.light_level } else { 0.0 };
    if light > 0.5 { Ink { on: "#2b1606", gh: 0.07, inv: "#f6a24a" } } else { Ink { on: "#17180f", gh: 0.055, inv: "#b4bba2" } }
}

/// Opens the layout's coordinate system: scaled to fill the LCD height, centred across it.
fn layout_open(l: &mut Lcd, d: &Device) {
    let (lx, ly, lw, lh) = LCD_RECT;
    let s = lh / GLASS.1;
    let ox = lx + (lw - GLASS.0 * s) / 2.0;
    let _ = write!(l.o, r#"<g transform="translate({ox:.3},{ly}) scale({s:.5}) translate({},{})">"#, -ORIGIN.0, -ORIGIN.1);
    if d.ink_level < 1.0 {
        let _ = write!(l.o, r#"<g opacity="{:.3}">"#, d.ink_level);
    } else {
        l.o += "<g>";
    }
}

/// The LCD: background and every segment except the pendulum fan (see `fan`), in panel
/// coordinates. It changes only when the display changes, not while the needle swings.
pub fn lcd(d: &Device) -> String {
    let light = if d.backlight && d.power { d.light_level } else { 0.0 };
    let mut l = Lcd { o: String::new(), ink: ink(d), power: d.power };
    let (lx, ly, lw, lh) = LCD_RECT;
    let _ = write!(l.o, r#"<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" fill="url(#lcdOff)"/>"#);
    if light > 0.0 {
        let _ = write!(l.o, r#"<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" fill="url(#lcdOn)" opacity="{light:.3}"/>"#);
    }
    layout_open(&mut l, d);
    content(&mut l, d);
    l.o += "</g></g>";
    doc(&l.o, &["lcdOn", "lcdOff"], None)
}

/// The pendulum fan, on a transparent layer over `lcd`: redrawn while the needle swings, so it
/// is kept small.
pub fn fan(d: &Device) -> String {
    let mut l = Lcd { o: String::new(), ink: ink(d), power: d.power };
    layout_open(&mut l, d);
    l.fan(if d.mode == Mode::Tune { None } else { Some(d.pend) });
    l.o += "</g></g>";
    doc(&l.o, &[], None)
}

/// The glass sheen over the whole LCD; it never changes.
pub fn glass() -> String {
    let (lx, ly, lw, lh) = LCD_RECT;
    doc(&format!(r#"<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" fill="url(#glass)"/>"#), &["glass"], None)
}

/// The panel area the fan covers: (x, y, w, h).
pub fn fan_rect() -> (f64, f64, f64, f64) {
    let (lx, ly, lw, lh) = LCD_RECT;
    let s = lh / GLASS.1;
    let ox = lx + (lw - GLASS.0 * s) / 2.0;
    // the outermost bars reach x 62 and 837, the peaks of the middle bars y 230, the feet y 549
    let (x0, x1, y0, y1) = (62.0, 837.0, 230.0, 550.0);
    let p = |x: f64, y: f64| (ox + (x - ORIGIN.0) * s, ly + (y - ORIGIN.1) * s);
    let ((ax, ay), (bx, by)) = (p(x0, y0), p(x1, y1));
    (ax.floor(), ay.floor(), (bx - ax).ceil() + 1.0, (by - ay).ceil() + 1.0)
}

fn content(l: &mut Lcd, d: &Device) {
    // A setting being changed blinks its label; the value stays lit.
    let hide = |e: Edit| d.edit == Some(e) && !d.blink;
    let mode = d.mode;
    let (metro, tune, coach, looping) = (mode == Mode::Metro, mode == Mode::Tune, mode == Mode::Coach, mode == Mode::Loop);

    // top left: memory, or the Rhythm Coach's trigger sensitivity
    l.text("TRIG", 60.0, 88.0, 28.0, 66.0, coach);
    l.text("TYPE", 61.0, 118.0, 27.0, 65.0, false);
    l.text("INST", 58.0, 145.0, 25.0, 70.0, false);
    l.text("SENS", 54.0, 170.0, 24.0, 66.0, coach && !hide(Edit::Sens));
    l.text("MEMORY", 131.0, 75.0, 25.0, 113.0, !coach && !hide(Edit::Memory) && !hide(Edit::Write));
    let n = if coach {
        d.sens
    } else if d.edit == Some(Edit::Write) {
        d.write_target
    } else if tune {
        d.tone_memory
    } else {
        d.memory
    };
    let stored = !(d.write_flashing() && !d.fast_blink);
    l.digits(&MEM_XS, MEM_Y, MID, &n.to_string(), stored);
    l.text("EDIT", 133.0, 197.0, 24.0, 56.0, d.edited && !tune && !coach);

    // mode labels beside the tempo
    l.text("TEMPO", 253.0, 83.0, 25.0, 91.0, metro);
    l.text("LOOP", 270.0, 110.0, 25.0, 70.0, looping);
    l.text("NOTE", 264.0, 138.0, 26.0, 69.0, tune);
    l.poly(&[(224.0, 140.0), (326.0, 140.0), (326.0, 171.0), (224.0, 171.0)], coach);
    let _ = write!(
        l.o,
        r#"<text fill="{}" x="229" y="167" font-size="{}" textLength="92" lengthAdjust="spacingAndGlyphs">COACH</text>"#,
        l.ink.inv,
        f1(24.0 / 0.714)
    );
    l.text("MIDI", 271.0, 197.0, 26.0, 49.0, false);

    // big digits: tempo, note, loop repeats, coach exercise, or the Time Check result
    if tune {
        let nn = NOTE_NAMES[(d.note % 12) as usize];
        let oct = (d.note / 12) as i32 - 1;
        let on = !d.flash_off;
        // the note takes the middle and right digits, next to the sharp sign
        l.digit(BIG_XS[0], BIG_Y, BIG, ' ', false);
        l.digit(BIG_XS[1], BIG_Y, BIG, nn.chars().next().unwrap(), on);
        l.digit(BIG_XS[2], BIG_Y, BIG, char::from_digit(oct as u32, 10).unwrap_or(' '), on);
        hash(l, on && nn.len() > 1);
    } else if let (true, Some(tc)) = (coach, d.time_check) {
        // a 7-position timing indicator across the 3 big digits (3 = on the beat)
        let p = [(0, "f"), (0, "b"), (1, "f"), (1, "abf"), (1, "b"), (2, "f"), (2, "b")][tc as usize];
        let mut masks = [String::new(), String::new(), String::new()];
        masks[p.0] += p.1;
        masks[1] += "d";
        for (i, x) in BIG_XS.iter().enumerate() {
            l.digit_mask(*x, BIG_Y, BIG, &masks[i], true);
        }
        hash(l, false);
    } else {
        let txt = match mode {
            Mode::Loop => d.loop_repeats.to_string(),
            Mode::Coach if d.show_coach_name => ["tch", "qct", "Gud", "Sud"][d.coach as usize].to_string(),
            _ => d.tempo.to_string(),
        };
        l.digits(&BIG_XS, BIG_Y, BIG, &txt, !d.flash_off);
        hash(l, false);
    }
    l.text("[GOOD]", 394.0, 229.0, 25.0, 111.0, coach && d.time_check == Some(3));

    // right: BEAT / PATTERN / PITCH / GUIDE, the STYLE digits and COUNT
    let guide = coach && d.edit == Some(Edit::Guide);
    l.text("BEAT", 612.0, 117.0, 25.0, 66.0, !tune && !guide && !d.pattern_mode && !hide(Edit::Beat));
    l.text("STYLE", 693.0, 105.0, 29.0, 91.0, !tune);
    l.text("Hz", 794.0, 104.0, 27.0, 33.0, tune);
    l.text("PATTERN", 561.0, 143.0, 23.0, 107.0, !tune && !guide && d.pattern_mode && !hide(Edit::Beat));
    l.text("PITCH", 591.0, 168.0, 22.0, 74.0, tune && !hide(Edit::Pitch));
    l.text("GUIDE", 580.0, 195.0, 24.0, 83.0, guide && !hide(Edit::Guide));
    let (count, style, beat2) = if tune {
        (String::new(), d.pitch.to_string(), 0)
    } else {
        // the count digit follows the beat while playing and is blank when stopped
        let count = if d.playing { d.beat_pos.to_string() } else { String::new() };
        let (style, beat2) = if looping {
            (d.loop_count.to_string(), 0)
        } else if guide {
            (d.guide.to_string(), 0)
        } else if d.edit == Some(Edit::CountIn) || (d.count_in > 0 && !d.playing) {
            (format!("-{}", d.count_in), 0)
        } else if d.pattern_mode {
            (d.pattern.to_string(), 0)
        } else {
            (d.beat1.to_string(), d.beat2)
        };
        (count, style, beat2)
    };
    if tune {
        // the pitch (e.g. 440) takes the count digit and both style digits
        l.digits(&STYLE_XS[..3], STYLE_Y, SMALL, &style, true);
    } else {
        l.digits(&STYLE_XS[..1], STYLE_Y, SMALL, &count, true);
        l.digits(&STYLE_XS[1..3], STYLE_Y, SMALL, &style, true);
    }
    let (px, py) = PLUS;
    l.poly(&[(px - 9.0, py - 2.5), (px + 9.0, py - 2.5), (px + 9.0, py + 2.5), (px - 9.0, py + 2.5)], beat2 > 0);
    l.poly(&[(px - 2.5, py - 9.0), (px + 2.5, py - 9.0), (px + 2.5, py + 9.0), (px - 2.5, py + 9.0)], beat2 > 0);
    l.digit(STYLE_XS[3], STYLE_Y, SMALL, char::from_digit(beat2 as u32, 10).filter(|_| beat2 > 0).unwrap_or(' '), true);
    l.poly(&[(682.0, 201.0), (694.0, 216.0), (670.0, 216.0)], !tune);
    let count_in = d.count_in > 0 && !looping && !tune;
    l.text("COUNT", 635.0, 245.0, 26.0, 91.0, !tune && !(count_in && hide(Edit::CountIn)));
    l.text("IN", 735.0, 244.0, 26.0, 26.0, count_in && !hide(Edit::CountIn));
    l.text("MUTE", 757.0, 292.0, 27.0, 79.0, if tune { !d.tone_on } else { d.muted });

    // voice, and the Rhythm Coach's trigger lamp
    let voice = metro && !d.pattern_mode;
    l.text("VOICE", 53.0, 229.0, 28.0, 86.0, voice && !hide(Edit::Voice));
    l.digit(VOICE_XY.0, VOICE_XY.1, MID, char::from_digit(d.voice as u32, 10).unwrap_or(' '), voice);
    l.text("TRIG", 183.0, 229.0, 28.0, 67.0, coach);
    let f = l.fill(coach && d.trig);
    let _ = write!(l.o, r#"<circle {f} cx="218" cy="253" r="18"/>"#);

    // the two icons that never light (the pendulum fan is its own layer: `fan`)
    l.sun();
    l.amp();

    // bottom: divider, rhythm levels, master volume
    l.poly(&[(54.0, 552.0), (846.0, 552.0), (846.0, 557.0), (54.0, 557.0)], true);
    let beat_icons = !tune && !d.pattern_mode;
    l.text("ACCENT", 54.0, 610.0, 25.0, 104.0, beat_icons);
    l.icons_rhythm(beat_icons);
    let lv = if tune { [0; 5] } else { d.shown_levels() };
    for (i, x) in WEDGE_XS.iter().enumerate() {
        l.wedge(*x, lv[i]);
    }
    let pat = !tune && d.pattern_mode;
    for (t, x, w) in [("KICK", 70.0, 68.0), ("SNARE", 192.0, 99.0), ("HH", 361.0, 39.0), ("OTHER1", 465.0, 108.0), ("OTHER2", 598.0, 119.0)] {
        l.text(t, x, 707.0, 27.0, w, pat);
    }
    l.text("VOL", 771.0, 587.0, 25.0, 51.0, true);
    let v = (d.sliders[5] as f64 / 20.0).ceil() as usize;
    for (i, y) in VOL_BARS.iter().enumerate() {
        l.poly(&[(767.0, *y), (826.0, *y), (826.0, y + 20.0), (767.0, y + 20.0)], i < v);
    }
}

/// The sharp sign after a note name.
fn hash(l: &mut Lcd, lit: bool) {
    l.poly(&[(586.0, 51.0), (592.0, 51.0), (584.0, 101.0), (578.0, 101.0)], lit);
    l.poly(&[(598.0, 51.0), (604.0, 51.0), (596.0, 101.0), (590.0, 101.0)], lit);
    l.poly(&[(578.0, 63.0), (604.0, 63.0), (603.0, 69.0), (577.0, 69.0)], lit);
    l.poly(&[(575.0, 83.0), (601.0, 83.0), (600.0, 89.0), (574.0, 89.0)], lit);
}
