//! The 30 built-in drum patterns.
//!
//! Each row is one bar; one character per step: `X` accent, `x` normal, `o` soft, `.` rest.
//! Rows map to the five rhythm sliders: KICK, SNARE, HH, OTHER1, OTHER2.

use super::synth::Inst;

pub struct Pattern {
    /// spec name (kept for reference; the LCD shows only the number)
    #[allow(dead_code)]
    pub name: &'static str,
    pub beats: u8,
    /// steps per beat: 4 (straight sixteenths) or 3 (triplet / shuffle feel)
    pub spb: u8,
    pub inst: [Inst; 5],
    pub rows: [&'static str; 5],
}

impl Pattern {
    /// Whether the instrument on this row plays at all in the pattern.
    pub fn uses(&self, row: usize) -> bool {
        self.rows[row].bytes().any(|b| b != b'.')
    }

    pub fn hit(&self, row: usize, step: usize) -> f32 {
        match self.rows[row].as_bytes().get(step) {
            Some(b'X') => 1.0,
            Some(b'x') => 0.7,
            Some(b'o') => 0.42,
            _ => 0.0,
        }
    }
}

use Inst::*;
const KIT: [Inst; 5] = [Kick, Snare, HatC, TomL, HatO];

const fn p(name: &'static str, beats: u8, spb: u8, inst: [Inst; 5], rows: [&'static str; 5]) -> Pattern {
    Pattern { name, beats, spb, inst, rows }
}

const NONE16: &str = "................";
const NONE12: &str = "............";

pub const PATTERNS: [Pattern; 30] = [
    p("8-BEAT 1", 4, 4, KIT, ["X.......X.......", "....X.......X...", "X.x.X.x.X.x.X.x.", NONE16, NONE16]),
    p("8-BEAT 2", 4, 4, KIT, ["X.....x.X.......", "....X.......X...", "x.x.x.x.x.x.x.x.", NONE16, "..............o."]),
    p("8-BEAT 3", 4, 4, KIT, ["X.......X.x.....", "....X.......X..o", "X.x.X.x.X.x.X.x.", NONE16, NONE16]),
    p("8-BEAT 4", 4, 4, KIT, ["X..x..x.X.......", "....X.......X...", "x.x.x.x.x.x.x.x.", "............x.x.", NONE16]),
    p("16-BEAT 1", 4, 4, KIT, ["X.....x...x.....", "....X.......X...", "xoxoxoxoxoxoxoxo", NONE16, NONE16]),
    p("16-BEAT 2", 4, 4, KIT, ["X..x..x...x..x..", "....X..o....X...", "xoxoxoxoxoxoxoxo", NONE16, "......o........."]),
    p("16-BEAT 3", 4, 4, KIT, ["X.x...x.X.x.....", "....X.......X.o.", "xoxoXoxoxoxoXoxo", NONE16, NONE16]),
    p("SHUFFLE 1", 4, 3, KIT, ["X.....X.....", "...X.....X..", "X.xX.xX.xX.x", NONE12, NONE12]),
    p("SHUFFLE 2", 4, 3, KIT, ["X....xX.....", "...X.....X.o", "x.xx.xx.xx.x", NONE12, NONE12]),
    p("SHUFFLE 3", 4, 3, [Kick, Snare, HatC, Rim, Ride], ["X.....X..x..", "...X.....X..", NONE12, NONE12, "X.xX.xX.xX.x"]),
    p("FUNK 1", 4, 4, KIT, ["X.x...x...x..x..", "....X..o.o..X..o", "xoxoxoxoxoxoxoxo", NONE16, NONE16]),
    p("FUNK 2", 4, 4, KIT, ["X..x.xx...x.....", "....X......oX.o.", "xoxoxoxoxoxoxoxo", NONE16, "..............x."]),
    p("JAZZ 1", 4, 3, [Kick, Snare, HatC, Rim, Ride], ["o..o..o..o..", "........o...", "...x.....x..", NONE12, "X..x.xX..x.x"]),
    p("JAZZ 2", 4, 3, [Kick, Snare, HatC, Rim, Ride], ["o.....o.....", "......o..o.o", "...x.....x..", NONE12, "X..X.xX..X.x"]),
    p("BLUES", 4, 3, KIT, ["X.....X.....", "...X.....X..", "x.xx.xx.xx.x", NONE12, NONE12]),
    p("TECHNO", 4, 4, [Kick, Clap, HatO, HatC, Shaker], ["X...X...X...X...", "....X.......X...", "..x...x...x...x.", "oxooo.oxooo.oxoo", NONE16]),
    p("HOUSE", 4, 4, [Kick, Clap, HatO, Shaker, HatC], ["X...X...X...X...", "....X.......X...", "..x...x...x...x.", "xoxoxoxoxoxoxoxo", NONE16]),
    p("COUNTRY", 4, 4, KIT, ["X.......X.......", "..o.X.o...o.X.o.", "x.x.x.x.x.x.x.x.", NONE16, NONE16]),
    p("REGGAE", 4, 4, [Kick, Rim, HatC, TomL, HatO], ["........X.......", "........X.......", "..x...x...x...x.", NONE16, NONE16]),
    p("CLAVE 3-2", 4, 4, [Kick, Snare, Shaker, Conga, Clave], ["X.......X.......", NONE16, "xoxoxoxoxoxoxoxo", "......x.......x.", "X..X..X...X.X..."]),
    p("CLAVE 2-3", 4, 4, [Kick, Snare, Shaker, Conga, Clave], ["X.......X.......", NONE16, "xoxoxoxoxoxoxoxo", "......x.......x.", "..X.X...X..X..X."]),
    p("SALSA", 4, 4, [Kick, Clave, Shaker, Cowbell, Conga], ["...x.......x....", "..X.X...X..X..X.", NONE16, "X.x.X.x.X.x.X.x.", "......xx......xx"]),
    p("RUMBA", 4, 4, [Kick, Clave, Shaker, Conga, TomH], ["X.......X.......", "X..X...X..X.X...", "xoxoxoxoxoxoxoxo", "......x.......xx", NONE16]),
    p("BOSSA NOVA", 4, 4, [Kick, Rim, HatC, TomL, Shaker], ["X..xX..xX..xX..x", "X..X..X...X..X..", "oooooooooooooooo", NONE16, NONE16]),
    p("SAMBA", 4, 4, [Kick, Snare, Shaker, TomL, Cowbell], ["X..xX..xX..xX..x", "....o.......o...", "xoxxxoxxxoxxxoxx", NONE16, "X.x.x.X.x.x.X.x."]),
    p("WALTZ", 3, 4, KIT, ["X...........", "....x...x...", "x...x...x...", NONE12, NONE12]),
    p("TANGO", 4, 4, KIT, ["X...X...X...X.x.", "............X...", "x...x...x...x...", NONE16, NONE16]),
    p("MAMBO", 4, 4, [Kick, Clave, Shaker, Conga, Cowbell], ["...x.......x....", "..X.X...X..X..X.", NONE16, "......x.......xx", "X.xXx.xXX.xXx.xX"]),
    p("CHACHACHA", 4, 4, [Kick, Snare, Shaker, Conga, Cowbell], ["X.......X.....x.", "............x.x.", "X.xxX.xxX.xxX.xx", "......x.......x.", "X...X...X...X..."]),
    p("MARCH", 4, 4, KIT, ["X.......X.......", "x.xxx.x.x.xxx.x.", NONE16, NONE16, NONE16]),
];
