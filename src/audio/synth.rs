//! Sound synthesis: oscillators, noise and filters, the click voices and the drum kit.

use std::f32::consts::TAU;

#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    fn new(kind: u8, f: f32, q: f32, sr: f32) -> Self {
        let w = TAU * (f / sr).min(0.49);
        let (s, c) = w.sin_cos();
        let a = s / (2.0 * q);
        let (b0, b1, b2) = match kind {
            0 => (a, 0.0, -a),                                    // band-pass
            _ => ((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0), // high-pass
        };
        let a0 = 1.0 + a;
        Biquad { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: -2.0 * c / a0, a2: (1.0 - a) / a0, z1: 0.0, z2: 0.0 }
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Wave {
    Sine,
    Tri,
    Square,
    Noise,
}

/// One oscillator/noise component with a pitch sweep, exponential decay and optional filter.
#[derive(Clone, Copy)]
struct Comp {
    wave: Wave,
    freq: f32,
    freq_end: f32,
    sweep: f32,
    amp: f32,
    decay: f32,
    phase: f32,
    filter: Option<Biquad>,
}

impl Comp {
    fn new(wave: Wave, freq: f32, amp: f32, decay_s: f32, sr: f32) -> Self {
        Comp {
            wave,
            freq,
            freq_end: freq,
            sweep: 1.0,
            amp,
            decay: (-1.0 / (decay_s * sr)).exp(),
            phase: 0.0,
            filter: None,
        }
    }
    fn sweep_to(mut self, f: f32, t: f32, sr: f32) -> Self {
        self.freq_end = f;
        self.sweep = (-1.0 / (t * sr)).exp();
        self
    }
    fn filt(mut self, b: Biquad) -> Self {
        self.filter = Some(b);
        self
    }
    fn run(&mut self, sr: f32, rng: &mut u32) -> f32 {
        let x = match self.wave {
            Wave::Sine => (self.phase * TAU).sin(),
            Wave::Tri => 4.0 * (self.phase - 0.5).abs() - 1.0,
            Wave::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            Wave::Noise => {
                *rng ^= *rng << 13;
                *rng ^= *rng >> 17;
                *rng ^= *rng << 5;
                (*rng as f32 / u32::MAX as f32) * 2.0 - 1.0
            }
        };
        self.phase = (self.phase + self.freq / sr).fract();
        self.freq = self.freq_end + (self.freq - self.freq_end) * self.sweep;
        let y = match &mut self.filter {
            Some(f) => f.run(x),
            None => x,
        };
        let out = y * self.amp;
        self.amp *= self.decay;
        out
    }
}

pub(super) struct Voice {
    comps: [Option<Comp>; 3],
    gain: f32,
    pub(super) age: u32,
    pub(super) len: u32,
    attack: u32,
}

impl Voice {
    fn new(gain: f32, len_s: f32, sr: f32, comps: &[Comp]) -> Self {
        let mut c = [None; 3];
        for (i, x) in comps.iter().take(3).enumerate() {
            c[i] = Some(*x);
        }
        Voice { comps: c, gain, age: 0, len: (len_s * sr) as u32, attack: (0.0015 * sr) as u32 }
    }
    pub(super) fn run(&mut self, sr: f32, rng: &mut u32) -> f32 {
        let mut s = 0.0;
        for c in self.comps.iter_mut().flatten() {
            s += c.run(sr, rng);
        }
        let a = if self.age < self.attack { self.age as f32 / self.attack as f32 } else { 1.0 };
        self.age += 1;
        s * self.gain * a
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Inst {
    Kick,
    Snare,
    Clap,
    HatC,
    HatO,
    Rim,
    TomL,
    TomH,
    Cowbell,
    Clave,
    Conga,
    Ride,
    Shaker,
}

/// What a click is for: picks the pitch within a voice.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Hit {
    Accent,
    Beat,
    Sub,
}

pub(super) fn click(voice: u8, hit: Hit, gain: f32, sr: f32) -> Voice {
    let f = match hit {
        Hit::Accent => 1.6,
        Hit::Beat => 1.0,
        Hit::Sub => 0.78,
    };
    match voice {
        // wind-up metronome: woody tick
        1 => Voice::new(gain, 0.06, sr, &[
            Comp::new(Wave::Noise, 0.0, 2.6, 0.009, sr).filt(Biquad::new(0, 2400.0 * f, 5.0, sr)),
            Comp::new(Wave::Sine, 620.0 * f, 0.35, 0.012, sr),
        ]),
        // electronic metronome: clean beep
        2 => Voice::new(gain, 0.09, sr, &[Comp::new(Wave::Sine, 1050.0 * f, 0.8, 0.028, sr)]),
        // electronic click: very short, stays clear at high tempos
        _ => Voice::new(gain, 0.02, sr, &[
            Comp::new(Wave::Square, 2100.0 * f, 0.28, 0.004, sr),
            Comp::new(Wave::Noise, 0.0, 0.5, 0.002, sr).filt(Biquad::new(1, 3000.0, 0.7, sr)),
        ]),
    }
}

pub(super) fn coach_click(hit: Hit, gain: f32, sr: f32) -> Voice {
    let f = if hit == Hit::Accent { 1.5 } else if hit == Hit::Beat { 1.0 } else { 0.8 };
    Voice::new(gain, 0.06, sr, &[Comp::new(Wave::Tri, 1250.0 * f, 0.75, 0.018, sr)])
}

pub(super) fn count_click(gain: f32, sr: f32) -> Voice {
    Voice::new(gain, 0.05, sr, &[
        Comp::new(Wave::Square, 1800.0, 0.22, 0.01, sr),
        Comp::new(Wave::Sine, 900.0, 0.5, 0.02, sr),
    ])
}

pub(super) fn drum(inst: Inst, gain: f32, sr: f32) -> Voice {
    use Wave::*;
    let c = |w, f, a, d| Comp::new(w, f, a, d, sr);
    match inst {
        Inst::Kick => Voice::new(gain, 0.35, sr, &[c(Sine, 150.0, 1.1, 0.12).sweep_to(45.0, 0.03, sr), c(Noise, 0.0, 0.25, 0.003).filt(Biquad::new(0, 1500.0, 1.0, sr))]),
        Inst::Snare => Voice::new(gain, 0.22, sr, &[c(Noise, 0.0, 0.8, 0.06).filt(Biquad::new(1, 1400.0, 0.7, sr)), c(Tri, 190.0, 0.6, 0.05).sweep_to(160.0, 0.05, sr)]),
        Inst::Clap => Voice::new(gain, 0.2, sr, &[c(Noise, 0.0, 1.4, 0.045).filt(Biquad::new(0, 1300.0, 1.5, sr))]),
        Inst::HatC => Voice::new(gain, 0.06, sr, &[c(Noise, 0.0, 0.55, 0.014).filt(Biquad::new(1, 7500.0, 0.7, sr))]),
        Inst::HatO => Voice::new(gain, 0.3, sr, &[c(Noise, 0.0, 0.5, 0.09).filt(Biquad::new(1, 6500.0, 0.7, sr))]),
        Inst::Rim => Voice::new(gain, 0.05, sr, &[c(Tri, 1700.0, 0.6, 0.008), c(Noise, 0.0, 0.8, 0.005).filt(Biquad::new(0, 2500.0, 3.0, sr))]),
        Inst::TomL => Voice::new(gain, 0.4, sr, &[c(Sine, 180.0, 0.9, 0.16).sweep_to(110.0, 0.12, sr)]),
        Inst::TomH => Voice::new(gain, 0.3, sr, &[c(Sine, 260.0, 0.8, 0.12).sweep_to(190.0, 0.1, sr)]),
        Inst::Cowbell => Voice::new(gain, 0.25, sr, &[
            c(Square, 545.0, 0.25, 0.07).filt(Biquad::new(0, 800.0, 3.0, sr)),
            c(Square, 815.0, 0.25, 0.07).filt(Biquad::new(0, 800.0, 3.0, sr)),
        ]),
        Inst::Clave => Voice::new(gain, 0.08, sr, &[c(Sine, 2500.0, 0.7, 0.018)]),
        Inst::Conga => Voice::new(gain, 0.3, sr, &[c(Sine, 330.0, 0.8, 0.09).sweep_to(280.0, 0.04, sr)]),
        Inst::Ride => Voice::new(gain, 0.9, sr, &[
            c(Noise, 0.0, 0.35, 0.3).filt(Biquad::new(0, 5200.0, 1.2, sr)),
            c(Square, 3150.0, 0.06, 0.25).filt(Biquad::new(1, 2000.0, 0.7, sr)),
        ]),
        Inst::Shaker => Voice::new(gain, 0.07, sr, &[c(Noise, 0.0, 0.45, 0.022).filt(Biquad::new(0, 6000.0, 1.0, sr))]),
    }
}
