//! Sample-accurate sequencer: metronome, patterns, count-in, Rhythm Coach and Loop playback.

use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::time::Instant;

use super::{START_LEAD, TEMPO_MAX};
use super::patterns::PATTERNS;
use super::synth::{click, coach_click, count_click, drum, Hit, Voice};

/// Ticks per beat: LCM of 2, 3 and 4 so eighths, triplets and sixteenths all land on ticks.
const TPB: u32 = 12;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayMode {
    Metro,
    /// 0 Time Check, 1 Quiet Count, 2 Gradual Up/Down, 3 Step Up/Down
    Coach(u8),
    Loop,
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub tempo: f64,
    /// `None` = BEAT mode, `Some(i)` = PATTERN i (0-based)
    pub pattern: Option<u8>,
    pub beat1: u8,
    /// 0 = off
    pub beat2: u8,
    /// 1..=4
    pub voice: u8,
    /// accent, quarter, eighth, sixteenth, triplet (or kick, snare, hh, other1, other2): 0..1
    pub levels: [f32; 5],
    pub muted: bool,
    pub mode: PlayMode,
    pub count_in: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct LoopStep {
    pub params: Params,
    pub repeats: u16,
    pub memory: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Mark {
    /// audible frame position of the beat
    pub pos: u64,
    /// 1-based beat in bar; negative = count-in beat
    pub num: i32,
    pub accent: bool,
    /// beat length in frames at the time
    pub len: f64,
}

/// Snapshot the UI reads every frame.
#[derive(Clone, Debug, Default)]
pub struct Status {
    pub playing: bool,
    pub tempo: f64,
    pub sr: f64,
    pub now: f64,
    pub last: Option<Mark>,
    /// START: when it was pressed and when its first beat is heard (audible frames)
    pub start: Option<(f64, f64)>,
    pub loop_memory: u8,
    pub loop_count: u16,
}

pub struct Engine {
    pub(super) sr: f64,
    /// frames rendered so far
    pos: u64,
    cb_instant: Instant,
    cb_pos: u64,
    /// output latency in frames (reported by the host, or one buffer)
    latency: u64,
    /// the last START: (pressed, first beat heard), audible frames
    start_at: Option<(f64, f64)>,

    params: Params,
    pub(super) playing: bool,
    pub(super) tempo: f64,
    next_tick: f64,
    tick: u32,
    beat: u32,
    bar: u32,
    count: u8,

    coach_base: f64,
    pub(super) coach_max: f64,
    pub(super) coach_dir: f64,

    program: Vec<LoopStep>,
    loop_idx: usize,
    loop_count: u16,

    marks: VecDeque<Mark>,
    voices: Vec<Voice>,
    rng: u32,

    pub(super) tone_freq: Option<f32>,
    tone_phase: f32,
    tone_amp: f32,
    pub(super) master: f32,
}

impl Engine {
    pub(super) fn new(sr: f64) -> Self {
        Engine {
            sr,
            pos: 0,
            cb_instant: Instant::now(),
            cb_pos: 0,
            start_at: None,
            latency: 0,
            params: Params {
                tempo: 120.0,
                pattern: None,
                beat1: 4,
                beat2: 0,
                voice: 1,
                levels: [1.0, 1.0, 0.0, 0.0, 0.0],
                muted: false,
                mode: PlayMode::Metro,
                count_in: 0,
            },
            playing: false,
            tempo: 120.0,
            next_tick: 0.0,
            tick: 0,
            beat: 0,
            bar: 0,
            count: 0,
            coach_base: 120.0,
            coach_max: TEMPO_MAX as f64,
            coach_dir: 1.0,
            program: Vec::new(),
            loop_idx: 0,
            loop_count: 0,
            marks: VecDeque::with_capacity(16),
            voices: Vec::with_capacity(32),
            rng: 0x9e37_79b9,
            tone_freq: None,
            tone_phase: 0.0,
            tone_amp: 0.0,
            master: 0.64,
        }
    }

    pub(super) fn start(&mut self, params: Params, program: Vec<LoopStep>) {
        self.params = params;
        self.program = program;
        self.loop_idx = 0;
        self.loop_count = 0;
        if self.params.mode == PlayMode::Loop {
            match self.program.first() {
                Some(s) => self.params = loop_params(s.params, params),
                None => return,
            }
        }
        self.tempo = self.params.tempo;
        self.coach_base = self.tempo;
        self.coach_dir = 1.0;
        self.tick = 0;
        self.beat = 0;
        self.bar = 0;
        self.count = params.count_in;
        // aim the first beat at START_LEAD after the press as heard: the output delay is
        // subtracted, and it can't be earlier than the next frame to be rendered
        let pressed = self.now_frame();
        self.next_tick = (pressed + START_LEAD * self.sr - self.latency as f64).max(self.pos as f64);
        self.start_at = Some((pressed, self.next_tick + self.latency as f64));
        self.marks.clear();
        self.playing = true;
    }

    pub(super) fn update(&mut self, p: Params) {
        let tempo_changed = (p.tempo - self.params.tempo).abs() > 1e-9;
        let mode = self.params.mode;
        if mode == PlayMode::Loop && self.playing {
            // the loop program drives everything except the live mute
            self.params.muted = p.muted;
            return;
        }
        self.params = p;
        self.params.mode = mode;
        if tempo_changed {
            self.tempo = p.tempo;
            // in Coach mode the dial/TAP set the minimum; that resets the maximum
            self.coach_base = p.tempo;
            self.coach_max = TEMPO_MAX as f64;
            self.coach_dir = 1.0;
        }
    }

    fn bar_len(&self) -> u32 {
        match self.params.pattern {
            Some(p) => PATTERNS[p as usize].beats as u32,
            None if self.params.beat2 > 0 && self.bar % 2 == 1 => self.params.beat2 as u32,
            None => self.params.beat1.max(1) as u32,
        }
    }

    fn trigger(&mut self, v: Voice) {
        if self.voices.len() < 48 {
            self.voices.push(v);
        }
    }

    fn mark(&mut self, num: i32, accent: bool) {
        let len = self.sr * 60.0 / self.tempo;
        if self.marks.len() >= 16 {
            self.marks.pop_front();
        }
        self.marks.push_back(Mark { pos: self.next_tick as u64 + self.latency, num, accent, len });
    }

    fn do_tick(&mut self) {
        let sr = self.sr as f32;
        let t = self.tick;
        if self.count > 0 {
            if t == 0 {
                if !self.params.muted {
                    self.trigger(count_click(0.9, sr));
                }
                let n = -(self.count as i32);
                self.mark(n, true);
            }
            self.advance_tick();
            if self.tick == 0 {
                self.count -= 1;
            }
            return;
        }

        let lv = self.params.levels;
        let mut quiet = 1.0;
        if let PlayMode::Coach(1) = self.params.mode {
            if (self.bar / 4) % 2 == 1 {
                quiet = 0.12;
            }
        }
        let coach = matches!(self.params.mode, PlayMode::Coach(_));
        if !self.params.muted {
            if let Some(pi) = self.params.pattern {
                let pat = &PATTERNS[pi as usize];
                let tps = TPB / pat.spb as u32;
                if t % tps == 0 {
                    let step = (self.beat * pat.spb as u32 + t / tps) as usize;
                    for row in 0..5 {
                        let v = pat.hit(row, step) * lv[row] * quiet;
                        if v > 0.0 {
                            self.trigger(drum(pat.inst[row], v, sr));
                        }
                    }
                }
            } else {
                let hit = |h: Hit, g: f32, s: &mut Self| {
                    let g = g * quiet;
                    if g > 0.0 {
                        let v = if coach { coach_click(h, g, sr) } else { click(s.params.voice, h, g, sr) };
                        s.trigger(v);
                    }
                };
                match t {
                    0 if self.beat == 0 => hit(Hit::Accent, lv[0], self),
                    0 => hit(Hit::Beat, lv[1], self),
                    6 => hit(Hit::Sub, lv[2], self),
                    3 | 9 => hit(Hit::Sub, lv[3], self),
                    4 | 8 => hit(Hit::Sub, lv[4], self),
                    _ => {}
                }
            }
        }
        if t == 0 {
            let (b, acc) = (self.beat as i32 + 1, self.beat == 0);
            self.mark(b, acc);
        }
        self.advance_tick();
        if self.tick == 0 {
            self.beat_end();
        }
    }

    fn advance_tick(&mut self) {
        self.tick = (self.tick + 1) % TPB;
        self.next_tick += self.sr * 60.0 / self.tempo / TPB as f64;
    }

    fn beat_end(&mut self) {
        self.beat += 1;
        let pattern = self.params.pattern.is_some();
        // Gradual Up/Down: 1 BPM every two beats (every two bars with PATTERN)
        if self.params.mode == PlayMode::Coach(2) && !pattern && self.beat % 2 == 0 {
            self.bump(1.0);
        }
        if self.beat >= self.bar_len() {
            self.beat = 0;
            self.bar += 1;
            match self.params.mode {
                PlayMode::Coach(2) if pattern && self.bar % 2 == 0 => self.bump(1.0),
                // Step Up/Down: 5 BPM every four bars
                PlayMode::Coach(3) if self.bar % 4 == 0 => self.bump(5.0),
                PlayMode::Loop => self.loop_bar(),
                _ => {}
            }
        }
    }

    fn bump(&mut self, d: f64) {
        self.tempo += d * self.coach_dir;
        if self.tempo >= self.coach_max {
            self.tempo = self.coach_max;
            self.coach_dir = -1.0;
        }
        if self.tempo <= self.coach_base {
            self.tempo = self.coach_base;
            self.coach_dir = 1.0;
        }
    }

    fn loop_bar(&mut self) {
        self.loop_count += 1;
        if self.loop_count >= self.program[self.loop_idx].repeats {
            self.loop_idx = (self.loop_idx + 1) % self.program.len();
            self.loop_count = 0;
            let muted = self.params.muted;
            self.params = self.program[self.loop_idx].params;
            self.params.mode = PlayMode::Loop;
            self.params.muted = muted;
            self.tempo = self.params.tempo;
            self.bar = 0;
        }
    }

    /// `latency_s`: time from this callback until its first frame is heard (0 = unknown).
    pub(super) fn render(&mut self, out: &mut [f32], channels: usize, latency_s: f64) {
        self.cb_instant = Instant::now();
        self.cb_pos = self.pos;
        let frames = (out.len() / channels) as u64;
        self.latency = if latency_s > 0.0 && latency_s < 1.0 { (latency_s * self.sr) as u64 } else { frames };
        let sr = self.sr as f32;
        let tone_step = 1.0 / (0.01 * sr);
        for frame in out.chunks_mut(channels) {
            while self.playing && self.pos as f64 >= self.next_tick {
                self.do_tick();
            }
            let mut s = 0.0;
            let mut i = 0;
            while i < self.voices.len() {
                let v = &mut self.voices[i];
                s += v.run(sr, &mut self.rng);
                if v.age >= v.len {
                    self.voices.swap_remove(i);
                } else {
                    i += 1;
                }
            }
            // reference tone with a short fade in/out
            let target = if self.tone_freq.is_some() { 0.32 } else { 0.0 };
            self.tone_amp += (target - self.tone_amp).clamp(-tone_step, tone_step) * 0.3;
            if self.tone_amp > 1e-4 {
                let f = self.tone_freq.unwrap_or(440.0);
                let ph = self.tone_phase * TAU;
                s += self.tone_amp * (ph.sin() + 0.12 * (2.0 * ph).sin());
                self.tone_phase = (self.tone_phase + f / sr).fract();
            }
            let y = (s * self.master).tanh();
            frame.fill(y);
            self.pos += 1;
        }
    }

    /// Audible frame position now.
    fn now_frame(&self) -> f64 {
        self.at_frame(Instant::now())
    }

    fn at_frame(&self, t: Instant) -> f64 {
        let dt = if t >= self.cb_instant {
            t.duration_since(self.cb_instant).as_secs_f64()
        } else {
            -self.cb_instant.duration_since(t).as_secs_f64()
        };
        self.cb_pos as f64 + dt * self.sr - self.latency as f64
    }

    pub(super) fn status(&self) -> Status {
        let now = self.now_frame();
        let last = self.marks.iter().rev().find(|m| m.pos as f64 <= now).copied();
        Status {
            playing: self.playing,
            tempo: self.tempo,
            sr: self.sr,
            now,
            last,
            start: self.start_at,
            loop_memory: self.program.get(self.loop_idx).map(|s| s.memory).unwrap_or(0),
            loop_count: self.loop_count,
        }
    }

    /// Offset of `t` from the nearest metronome beat, in beats (>0 = late / behind the beat).
    pub(super) fn beat_offset(&self, t: Instant) -> Option<f64> {
        let f = self.at_frame(t);
        let m = self
            .marks
            .iter()
            .filter(|m| m.num > 0)
            .min_by(|a, b| (a.pos as f64 - f).abs().total_cmp(&(b.pos as f64 - f).abs()))?;
        let d = (f - m.pos as f64) / m.len;
        (d.abs() < 0.5).then_some(d)
    }
}

/// Loop playback takes rhythm settings from memory but keeps the live mute and count-in.
fn loop_params(mem: Params, live: Params) -> Params {
    Params { muted: live.muted, mode: PlayMode::Loop, count_in: live.count_in, ..mem }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(tempo: f64) -> Params {
        Params {
            tempo,
            pattern: None,
            beat1: 4,
            beat2: 0,
            voice: 1,
            levels: [1.0, 1.0, 0.0, 0.0, 0.0],
            muted: false,
            mode: PlayMode::Metro,
            count_in: 0,
        }
    }

    /// Renders `secs` of audio in 512-frame buffers and returns all beat marks.
    fn run(e: &mut Engine, secs: f64) -> Vec<Mark> {
        let mut out = Vec::new();
        let mut buf = vec![0.0f32; 512];
        let frames = (secs * e.sr) as usize;
        for _ in 0..frames / 512 {
            e.render(&mut buf, 1, 0.0);
            for m in e.marks.drain(..) {
                out.push(m);
            }
        }
        out
    }

    #[test]
    fn tempo_is_sample_accurate() {
        let mut e = Engine::new(48_000.0);
        e.start(params(120.0), vec![]);
        let m = run(&mut e, 5.0);
        assert!(m.len() >= 9);
        for w in m.windows(2) {
            let d = w[1].pos as i64 - w[0].pos as i64;
            assert!((d - 24_000).abs() <= 1, "beat spacing {d}");
        }
        let nums: Vec<i32> = m.iter().take(6).map(|m| m.num).collect();
        assert_eq!(nums, [1, 2, 3, 4, 1, 2]);
        assert!(m[0].accent && !m[1].accent && m[4].accent);
    }

    #[test]
    fn count_in_then_bar() {
        let mut e = Engine::new(48_000.0);
        e.start(Params { count_in: 3, ..params(240.0) }, vec![]);
        let nums: Vec<i32> = run(&mut e, 2.0).iter().take(5).map(|m| m.num).collect();
        assert_eq!(nums, [-3, -2, -1, 1, 2]);
    }

    #[test]
    fn beat2_alternates_bars() {
        let mut e = Engine::new(48_000.0);
        e.start(Params { beat1: 3, beat2: 2, ..params(240.0) }, vec![]);
        let nums: Vec<i32> = run(&mut e, 3.0).iter().take(8).map(|m| m.num).collect();
        assert_eq!(nums, [1, 2, 3, 1, 2, 1, 2, 3]);
    }

    #[test]
    fn gradual_up_one_bpm_every_two_beats() {
        let mut e = Engine::new(48_000.0);
        e.start(Params { mode: PlayMode::Coach(2), ..params(100.0) }, vec![]);
        run(&mut e, 60.0 / 100.0 * 10.5);
        assert!((e.tempo - 105.0).abs() < 0.01, "tempo {}", e.tempo);
    }

    #[test]
    fn step_up_five_bpm_every_four_bars() {
        let mut e = Engine::new(48_000.0);
        e.start(Params { mode: PlayMode::Coach(3), ..params(200.0) }, vec![]);
        // 16 beats at 200 = 4.8 s; a bit more to cross the bar line
        run(&mut e, 4.9);
        assert!((e.tempo - 205.0).abs() < 0.01, "tempo {}", e.tempo);
    }

    #[test]
    fn loop_plays_memories_in_order() {
        let mut e = Engine::new(48_000.0);
        let a = LoopStep { params: Params { beat1: 2, ..params(240.0) }, repeats: 2, memory: 1 };
        let b = LoopStep { params: Params { beat1: 3, ..params(240.0) }, repeats: 1, memory: 5 };
        e.start(Params { mode: PlayMode::Loop, ..params(120.0) }, vec![a, b]);
        let nums: Vec<i32> = run(&mut e, 3.0).iter().take(10).map(|m| m.num).collect();
        // memory 1: two bars of 2, memory 5: one bar of 3, then back to memory 1
        assert_eq!(nums, [1, 2, 1, 2, 1, 2, 3, 1, 2, 1]);
    }

    #[test]
    fn first_beat_is_heard_start_lead_after_the_press() {
        let mut e = Engine::new(48_000.0);
        // a running stream with small buffers and a known output delay (one buffer, as when the
        // host reports none), so the first beat can be scheduled START_LEAD after the press
        let mut buf = vec![0.0f32; 128];
        e.render(&mut buf, 1, 0.0);
        e.start(params(120.0), vec![]);
        let (pressed, first) = e.status().start.expect("start time");
        assert!((first - pressed - START_LEAD * 48_000.0).abs() < 1.0, "first beat {} frames after the press", first - pressed);
        // keep the same buffer size, so the output delay stays the same
        let mut marks = Vec::new();
        while marks.is_empty() {
            e.render(&mut buf, 1, 0.0);
            marks.extend(e.marks.drain(..));
        }
        assert!((marks[0].pos as f64 - first).abs() < 1.0, "first beat heard at {} instead of {first}", marks[0].pos);
    }

    #[test]
    fn output_is_bounded() {
        let mut e = Engine::new(48_000.0);
        e.start(Params { levels: [1.0; 5], pattern: Some(16), ..params(250.0) }, vec![]);
        let mut buf = vec![0.0f32; 4096];
        for _ in 0..100 {
            e.render(&mut buf, 2, 0.0);
            assert!(buf.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
        }
        assert!(buf.iter().any(|x| x.abs() > 0.01), "pattern should be audible");
    }
}
