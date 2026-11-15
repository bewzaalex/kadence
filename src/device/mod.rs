//! KD-90 behaviour: modes, buttons, dial, memories and the LCD state (port of the mockup logic).

mod store;

use std::time::{Duration, Instant};

use crate::audio::{Audio, LoopStep, Params, PlayMode, TEMPO_MAX, TEMPO_MIN};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Btn {
    Power,
    Light,
    Mute,
    Write,
    Memory,
    Midi,
    Tempo,
    Loop,
    Tune,
    Coach,
    Beat,
    Voice,
    Pitch,
    Trig,
    Tap,
    Start,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Metro,
    Loop,
    Tune,
    Coach,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Edit {
    Memory,
    Write,
    Beat,
    Voice,
    CountIn,
    Sens,
    Guide,
    Pitch,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Mem {
    pub tempo: u16,
    pub pattern_mode: bool,
    pub beat1: u8,
    pub beat2: u8,
    /// 1..=30
    pub pattern: u8,
    pub voice: u8,
    /// slider positions 0..=100
    pub sliders: [u8; 5],
    pub repeats: u16,
}

impl Default for Mem {
    fn default() -> Self {
        Mem { tempo: 120, pattern_mode: false, beat1: 4, beat2: 0, pattern: 1, voice: 1, sliders: [100, 100, 0, 0, 0], repeats: 0 }
    }
}

/// Factory memories: all default; only memory 1 takes part in Loop Play.
pub fn factory_memories() -> Vec<Mem> {
    let mut m = vec![Mem::default(); 50];
    m[0].repeats = 1;
    m
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ToneMem {
    /// MIDI note number, C2 (36) ..= B6 (95)
    pub note: u8,
    pub pitch: u16,
}

impl Default for ToneMem {
    fn default() -> Self {
        ToneMem { note: 69, pitch: 440 }
    }
}

/// Click voices the VOICE dial offers. Voice 4, the counting voice, is off until it is
/// redone (see docs/TODO.md); a memory that stored it plays voice 3.
pub const VOICES: u8 = 3;

/// Buttons whose functions are written but not yet tested and released: pressing one shows this
/// notice instead. Remove a button here to release its function.
fn unreleased(b: Btn) -> Option<&'static str> {
    match b {
        Btn::Loop => Some("Loop is not available yet"),
        Btn::Coach | Btn::Trig => Some("Rhythm Coach is not available yet"),
        _ => None,
    }
}

/// A slider position (0..=100) as a level on the LCD (0..=5).
fn level(slider: u8) -> u8 {
    (slider as f32 / 20.0).round() as u8
}

/// Pendulum positions on the LCD fan (0 = far left; the middle one is at rest).
pub const PEND_N: usize = 27;

/// Half a blink cycle: every blink on the LCD runs at 120 BPM (on and off once per 0.5 s).
pub const BLINK: Duration = Duration::from_millis(250);
/// POWER: the LCD goes dark at once, and the app quits this long after.
const POWER_OFF_TIME: Duration = Duration::from_millis(100);
/// TEMPO and TUNE: the tempo or note flashes twice within this time, then stays lit.
const ENTRY_FLASH: Duration = Duration::from_millis(1000);

pub struct Device {
    pub power: bool,
    pub backlight: bool,
    /// set when POWER was pressed; the LCD goes dark and the app quits soon after
    power_off: Option<Instant>,
    /// LCD backlight and segment strength: 1 normally, 0 once POWER was pressed
    pub light_level: f64,
    pub ink_level: f64,
    /// set by TEMPO and TUNE; the tempo or note flashes twice, then stays lit
    entry_flash: Option<Instant>,
    /// the tempo or note is hidden at this moment of the flash
    pub flash_off: bool,
    pub mode: Mode,
    pub tempo: u16,
    pub memory: u8,
    pub pattern_mode: bool,
    pub beat1: u8,
    pub beat2: u8,
    pub pattern: u8,
    pub voice: u8,
    pub count_in: u8,
    pub edit: Option<Edit>,
    pub blink: bool,
    pub fast_blink: bool,
    pub edited: bool,
    pub playing: bool,
    pub beat_pos: u8,
    pub muted: bool,
    /// 5 rhythm sliders + master VOL, 0..=100
    pub sliders: [u8; 6],
    /// the lit pendulum bar (0 = far left)
    pub pend: usize,
    /// beats heard since START (the pendulum swings the other way on each one)
    swings: u64,
    /// position of the last beat seen, to notice the next one
    swing_beat: Option<u64>,
    pub sens: u8,
    pub coach: u8,
    pub guide: u8,
    pub trig: bool,
    pub time_check: Option<u8>,
    pub show_coach_name: bool,
    pub note: u8,
    pub pitch: u16,
    pub tone_memory: u8,
    pub tone_on: bool,
    pub loop_repeats: u16,
    pub loop_count: u16,
    pub write_target: u8,
    /// tempo indicator LEDs: [accent (red), beat (green)]
    pub leds: [bool; 2],
    pub notice: Option<(&'static str, Instant)>,

    pub memories: Vec<Mem>,
    pub tones: Vec<ToneMem>,

    epoch: Instant,
    write_flash_until: Option<Instant>,
    trig_until: Option<Instant>,
    coach_name_until: Option<Instant>,
    tap_times: Vec<Instant>,
    pub held: Vec<Btn>,
    pub audio: Audio,
}

impl Device {
    pub fn new(audio: Audio) -> Self {
        let now = Instant::now();
        let saved = store::load();
        let mut d = Device {
            power: true,
            backlight: saved.backlight,
            power_off: None,
            light_level: 1.0,
            ink_level: 1.0,
            entry_flash: None,
            flash_off: false,
            mode: Mode::Metro,
            tempo: 120,
            memory: saved.memory.clamp(1, 50),
            pattern_mode: false,
            beat1: 4,
            beat2: 0,
            pattern: 1,
            voice: 1,
            count_in: 0,
            edit: None,
            blink: true,
            fast_blink: true,
            edited: false,
            playing: false,
            beat_pos: 1,
            muted: false,
            sliders: [100, 100, 0, 0, 0, 80],
            pend: PEND_N / 2,
            swings: 0,
            swing_beat: None,
            sens: saved.sens.clamp(1, 16),
            coach: 0,
            guide: saved.guide.clamp(1, 3),
            trig: false,
            time_check: None,
            show_coach_name: true,
            note: 69,
            pitch: 440,
            tone_memory: 1,
            tone_on: false,
            loop_repeats: 1,
            loop_count: 0,
            write_target: 1,
            leds: [false; 2],
            notice: None,
            memories: saved.memories,
            tones: saved.tones,
            epoch: now,
            write_flash_until: None,
            trig_until: None,
            coach_name_until: None,
            tap_times: Vec::new(),
            held: Vec::new(),
            audio,
        };
        d.load_memory(d.memory);
        d.edited = saved.edited;
        if let Some(cur) = saved.current {
            d.apply_mem(cur);
        }
        d.set_sens(d.sens);
        if d.audio.error.is_some() {
            d.notice("No audio output device: sound is off");
        }
        d.sync_audio();
        d
    }

    // ------------------------------------------------------------ helpers

    /// The rhythm sliders as the LCD's levels, 0..=5.
    pub fn levels(&self) -> [u8; 5] {
        std::array::from_fn(|i| level(self.sliders[i]))
    }

    /// The levels the LCD shows: in a pattern, an instrument the pattern doesn't use shows no
    /// level, whatever its slider says.
    pub fn shown_levels(&self) -> [u8; 5] {
        let mut l = self.levels();
        if self.pattern_mode {
            let pat = &crate::audio::patterns::PATTERNS[self.pattern as usize - 1];
            for (row, v) in l.iter_mut().enumerate() {
                if !pat.uses(row) {
                    *v = 0;
                }
            }
        }
        l
    }

    /// What the audio engine plays now.
    fn params(&self) -> Params {
        let mode = match self.mode {
            Mode::Coach => PlayMode::Coach(self.coach),
            Mode::Loop => PlayMode::Loop,
            _ => PlayMode::Metro,
        };
        Params { mode, ..self.mem_params(&self.current_mem()) }
    }

    /// What the audio engine plays for memory `m` in Loop playback.
    fn mem_params(&self, m: &Mem) -> Params {
        Params {
            tempo: m.tempo as f64,
            pattern: m.pattern_mode.then(|| m.pattern - 1),
            beat1: m.beat1,
            beat2: m.beat2,
            voice: m.voice,
            levels: m.sliders.map(|v| level(v) as f32 / 5.0),
            muted: self.muted,
            mode: PlayMode::Loop,
            count_in: self.count_in,
        }
    }

    pub fn sync_audio(&self) {
        self.audio.update(self.params());
        self.audio.set_master(self.sliders[5] as f32 / 100.0);
    }

    fn current_mem(&self) -> Mem {
        Mem {
            tempo: self.tempo,
            pattern_mode: self.pattern_mode,
            beat1: self.beat1,
            beat2: self.beat2,
            pattern: self.pattern,
            voice: self.voice,
            sliders: [self.sliders[0], self.sliders[1], self.sliders[2], self.sliders[3], self.sliders[4]],
            repeats: self.memories[self.memory as usize - 1].repeats,
        }
    }

    fn apply_mem(&mut self, m: Mem) {
        self.tempo = m.tempo.clamp(TEMPO_MIN, TEMPO_MAX);
        self.pattern_mode = m.pattern_mode;
        self.beat1 = m.beat1.clamp(1, 9);
        self.beat2 = m.beat2.min(9);
        self.pattern = m.pattern.clamp(1, 30);
        self.voice = m.voice.clamp(1, VOICES);
        self.sliders[..5].copy_from_slice(&m.sliders);
        self.loop_repeats = m.repeats;
    }

    fn load_memory(&mut self, n: u8) {
        let m = self.memories[n as usize - 1];
        self.apply_mem(m);
        self.edited = false;
    }

    fn edit_changed(&mut self) {
        self.edited = true;
    }

    pub fn save(&self) {
        store::save(&store::Saved {
            memories: self.memories.clone(),
            tones: self.tones.clone(),
            memory: self.memory,
            sens: self.sens,
            guide: self.guide,
            current: Some(self.current_mem()),
            edited: self.edited,
            backlight: self.backlight,
        });
    }

    /// The microphone's trigger sensitivity, 1..=16, here and in the onset detector.
    fn set_sens(&mut self, v: u8) {
        self.sens = v;
        self.audio.sens.store(v as u32, std::sync::atomic::Ordering::Relaxed);
    }

    fn notice(&mut self, msg: &'static str) {
        self.notice = Some((msg, Instant::now()));
    }

    fn set_tone(&mut self, on: bool) {
        self.tone_on = on;
        let f = self.pitch as f32 * 2f32.powf((self.note as f32 - 69.0) / 12.0);
        self.audio.tone(on.then_some(f));
    }

    // ------------------------------------------------------------ transport

    fn start(&mut self) {
        (self.swings, self.swing_beat) = (0, None);
        let mut program = Vec::new();
        if self.mode == Mode::Loop {
            for (i, m) in self.memories.iter().enumerate() {
                if m.repeats > 0 {
                    program.push(LoopStep { params: self.mem_params(m), repeats: m.repeats, memory: i as u8 + 1 });
                }
            }
            if program.is_empty() {
                return;
            }
        }
        self.audio.start(self.params(), program);
        self.playing = true;
        self.show_coach_name = false;
        if self.mode == Mode::Coach {
            self.audio.ensure_mic();
        }
    }

    fn stop(&mut self) {
        self.audio.stop();
        self.playing = false;
        self.pend = PEND_N / 2;
        self.beat_pos = 1;
        self.leds = [false; 2];
        if self.mode == Mode::Coach {
            self.show_coach_name = true;
        }
        if self.mode == Mode::Loop {
            self.load_memory(self.memory);
            self.loop_count = 0;
        }
    }

    fn set_mode(&mut self, m: Mode) {
        // another screen: a flash still running on the old one ends
        self.entry_flash = None;
        if self.playing {
            self.stop();
        }
        if self.mode == Mode::Tune && m != Mode::Tune {
            self.set_tone(false);
        }
        self.mode = m;
        self.edit = None;
        if m == Mode::Coach {
            self.audio.ensure_mic();
        }
        self.sync_audio();
    }

    // ------------------------------------------------------------ input

    pub fn button_down(&mut self, b: Btn, now: Instant) {
        if !self.held.contains(&b) {
            self.held.push(b);
        }
        self.press(b, now);
    }

    pub fn button_up(&mut self, b: Btn, now: Instant) {
        let _ = now;
        self.held.retain(|x| *x != b);
    }

    pub fn is_held(&self, b: Btn) -> bool {
        self.held.contains(&b)
    }

    fn press(&mut self, b: Btn, now: Instant) {
        if !self.power || self.power_off.is_some() {
            return;
        }
        // A setting stays open (its label blinking) until another button is pressed. START/STOP
        // and LIGHT leave it open, so it can be changed while the metronome plays.
        let prev = self.edit;
        if !matches!(b, Btn::Start | Btn::Light) {
            self.edit = None;
        }
        // Loop playback and the Rhythm Coach are not released yet: their buttons only say so
        // (the code behind them stays; see docs/TODO.md)
        if let Some(msg) = unreleased(b) {
            return self.notice(msg);
        }
        let combo = (b == Btn::Beat && self.is_held(Btn::Voice)) || (b == Btn::Voice && self.is_held(Btn::Beat));
        match b {
            Btn::Power => self.power_button(now),
            Btn::Light => {
                self.backlight = !self.backlight;
                self.save();
            }
            Btn::Mute => {
                if self.mode == Mode::Tune {
                    self.set_tone(!self.tone_on);
                } else {
                    self.muted = !self.muted;
                    self.sync_audio();
                }
            }
            Btn::Memory => {
                if !(self.mode == Mode::Loop && self.playing) {
                    self.edit = if prev == Some(Edit::Memory) { None } else { Some(Edit::Memory) };
                }
            }
            Btn::Write => self.write(now, prev == Some(Edit::Write)),
            Btn::Midi => self.notice("MIDI input is not implemented yet"),
            Btn::Tempo => {
                if self.mode != Mode::Metro {
                    self.set_mode(Mode::Metro);
                }
                self.entry_flash = Some(now);
            }
            Btn::Loop => {
                self.set_mode(Mode::Loop);
                self.loop_repeats = self.memories[self.memory as usize - 1].repeats;
                self.loop_count = 0;
            }
            Btn::Tune => {
                if self.mode != Mode::Tune {
                    self.set_mode(Mode::Tune);
                    let t = self.tones[self.tone_memory as usize - 1];
                    self.note = t.note;
                    self.pitch = t.pitch;
                }
                // like TEMPO: every press flashes the note, also when already on the tuner
                self.entry_flash = Some(now);
            }
            Btn::Pitch => {
                if self.mode != Mode::Tune {
                    self.press(Btn::Tune, now);
                }
                self.edit = Some(Edit::Pitch);
            }
            Btn::Coach => {
                if self.mode == Mode::Coach && self.playing {
                    self.stop();
                } else if self.mode != Mode::Coach {
                    self.set_mode(Mode::Coach);
                    self.coach = 0;
                } else {
                    self.coach = (self.coach + 1) % 4;
                    self.sync_audio();
                }
                self.show_coach_name = true;
                self.time_check = None;
            }
            Btn::Trig => {
                if self.mode != Mode::Coach {
                    self.set_mode(Mode::Coach);
                    self.coach = 0;
                    self.show_coach_name = true;
                    self.edit = Some(Edit::Sens);
                } else if self.playing {
                    self.stop();
                } else {
                    self.edit = match prev {
                        Some(Edit::Sens) => Some(Edit::Guide),
                        Some(Edit::Guide) => None,
                        _ => Some(Edit::Sens),
                    };
                }
            }
            Btn::Beat | Btn::Voice if combo => {
                // [BEAT/PATTERN] + [VOICE]: count-in on/off
                if self.count_in > 0 {
                    self.count_in = 0;
                } else {
                    self.count_in = 4;
                    self.edit = Some(Edit::CountIn);
                }
                self.sync_audio();
            }
            Btn::Beat => {
                if matches!(self.mode, Mode::Tune | Mode::Coach) {
                    self.set_mode(Mode::Metro);
                }
                // the first press opens the current mode; each further press switches BEAT <-> PATTERN
                if prev == Some(Edit::Beat) {
                    self.pattern_mode = !self.pattern_mode;
                    self.edit_changed();
                    self.sync_audio();
                }
                self.edit = Some(Edit::Beat);
            }
            Btn::Voice => {
                if matches!(self.mode, Mode::Tune | Mode::Coach) {
                    self.set_mode(Mode::Metro);
                }
                if self.pattern_mode {
                    self.pattern_mode = false;
                    self.edit_changed();
                    self.sync_audio();
                }
                self.edit = Some(Edit::Voice);
            }
            Btn::Tap => self.tap(now),
            Btn::Start => {
                if prev == Some(Edit::Write) {
                    self.edit = None;
                } else if self.mode == Mode::Tune {
                    self.set_tone(!self.tone_on);
                } else if self.playing {
                    self.stop();
                } else {
                    self.start();
                }
            }
        }
    }

    /// POWER turns the app off: playback stops, the display goes dark and the app quits.
    /// Holding MEMORY while pressing POWER restores the factory settings instead.
    fn power_button(&mut self, now: Instant) {
        if self.is_held(Btn::Memory) {
            self.factory_reset();
            return;
        }
        self.shut_down(now);
    }

    /// Stops everything, saves, and starts the power-off: the LCD goes dark and the app quits.
    pub fn shut_down(&mut self, now: Instant) {
        if self.power_off.is_some() {
            return;
        }
        self.stop();
        self.set_tone(false);
        self.edit = None;
        self.leds = [false; 2];
        self.save();
        self.power_off = Some(now);
    }

    /// True once the power-off animation has finished and the app should quit.
    pub fn powered_off(&self) -> bool {
        !self.power
    }

    /// Restores factory settings and the start-up state.
    fn factory_reset(&mut self) {
        self.stop();
        self.set_tone(false);
        self.edit = None;
        self.mode = Mode::Metro;
        self.count_in = 0;
        self.muted = false;
        self.memories = factory_memories();
        self.tones = vec![ToneMem::default(); 10];
        self.memory = 1;
        self.tone_memory = 1;
        self.set_sens(8);
        self.guide = 2;
        self.sliders = [100, 100, 0, 0, 0, self.sliders[5]];
        self.load_memory(1);
        self.sync_audio();
        self.save();
        self.notice("Factory settings restored");
    }

    /// WRITE: the first press opens the target memory, the second stores (`open`: WRITE was open).
    fn write(&mut self, now: Instant, open: bool) {
        if self.mode == Mode::Coach && self.playing && self.coach >= 2 {
            // tempo in effect becomes the maximum metronome value
            self.audio.coach_set_max();
            return;
        }
        if open {
            let t = self.write_target;
            if self.mode == Mode::Tune {
                self.tones[t as usize - 1] = ToneMem { note: self.note, pitch: self.pitch };
                self.tone_memory = t;
            } else {
                let m = self.current_mem();
                self.memories[t as usize - 1] = Mem { repeats: self.memories[t as usize - 1].repeats, ..m };
                self.memory = t;
            }
            self.edited = false;
            self.edit = None;
            // the memory number flashes rapidly while storing
            self.write_flash_until = Some(now + Duration::from_millis(700));
            self.save();
        } else {
            self.edit = Some(Edit::Write);
            self.write_target = if self.mode == Mode::Tune { self.tone_memory } else { self.memory };
        }
    }

    fn tap(&mut self, now: Instant) {
        if !matches!(self.mode, Mode::Metro | Mode::Coach) {
            return;
        }
        self.tap_times.retain(|t| now.duration_since(*t) < Duration::from_millis(2100));
        self.tap_times.push(now);
        if self.tap_times.len() > 5 {
            self.tap_times.remove(0);
        }
        if self.tap_times.len() >= 2 {
            let span = now.duration_since(self.tap_times[0]).as_secs_f64();
            let avg = span / (self.tap_times.len() - 1) as f64;
            self.tempo = ((60.0 / avg).round() as u16).clamp(TEMPO_MIN, TEMPO_MAX);
            if self.mode == Mode::Metro {
                self.edit_changed();
            } else {
                self.show_coach_name = false;
            }
            self.sync_audio();
        }
    }

    /// TEMPO/VALUE dial: `n` detents (positive = clockwise).
    pub fn dial(&mut self, n: i32, now: Instant) {
        if !self.power || self.power_off.is_some() || n == 0 {
            return;
        }
        let add = |v: i32, lo: i32, hi: i32| (v + n).clamp(lo, hi);
        let wrap = |v: i32, lo: i32, hi: i32| (v - lo + n).rem_euclid(hi - lo + 1) + lo;
        match self.edit {
            Some(Edit::Memory) => {
                if self.mode == Mode::Tune {
                    self.tone_memory = wrap(self.tone_memory as i32, 1, 10) as u8;
                    let t = self.tones[self.tone_memory as usize - 1];
                    self.note = t.note;
                    self.pitch = t.pitch;
                    if self.tone_on {
                        self.set_tone(true);
                    }
                } else {
                    self.memory = wrap(self.memory as i32, 1, 50) as u8;
                    self.load_memory(self.memory);
                    self.sync_audio();
                }
                return;
            }
            Some(Edit::Write) => {
                let hi = if self.mode == Mode::Tune { 10 } else { 50 };
                self.write_target = wrap(self.write_target as i32, 1, hi) as u8;
                return;
            }
            Some(Edit::Beat) if self.pattern_mode => self.pattern = wrap(self.pattern as i32, 1, 30) as u8,
            Some(Edit::Beat) => {
                // 1 … 9, then the combined bars 1+1, 1+2 … 9+9, then 1 again
                let (b1, b2) = (self.beat1 as i32, self.beat2 as i32);
                let idx = if b2 == 0 { b1 - 1 } else { 9 + (b1 - 1) * 9 + (b2 - 1) };
                let idx = (idx + n).rem_euclid(90);
                (self.beat1, self.beat2) = if idx < 9 { (idx as u8 + 1, 0) } else { ((idx - 9) as u8 / 9 + 1, (idx - 9) as u8 % 9 + 1) };
            }
            Some(Edit::Voice) => self.voice = add(self.voice as i32, 1, VOICES as i32) as u8,
            Some(Edit::CountIn) => {
                self.count_in = add(self.count_in as i32, 1, 8) as u8;
                self.sync_audio();
                return;
            }
            Some(Edit::Sens) => {
                self.set_sens(add(self.sens as i32, 1, 16) as u8);
                return;
            }
            Some(Edit::Guide) => {
                self.guide = add(self.guide as i32, 1, 3) as u8;
                return;
            }
            Some(Edit::Pitch) => {
                self.pitch = add(self.pitch as i32, 438, 445) as u16;
                if self.tone_on {
                    self.set_tone(true);
                }
                return;
            }
            None => {
                match self.mode {
                    Mode::Tune => {
                        self.note = add(self.note as i32, 36, 95) as u8;
                        if self.tone_on {
                            self.set_tone(true);
                        }
                    }
                    Mode::Loop => {
                        if !self.playing {
                            self.loop_repeats = add(self.loop_repeats as i32, 0, 255) as u16;
                            self.memories[self.memory as usize - 1].repeats = self.loop_repeats;
                        }
                    }
                    Mode::Coach => {
                        self.tempo = add(self.tempo as i32, TEMPO_MIN as i32, TEMPO_MAX as i32) as u16;
                        self.show_coach_name = false;
                        self.coach_name_until = Some(now + Duration::from_millis(1500));
                        self.sync_audio();
                    }
                    Mode::Metro => {
                        self.tempo = add(self.tempo as i32, TEMPO_MIN as i32, TEMPO_MAX as i32) as u16;
                        self.edit_changed();
                        self.sync_audio();
                    }
                }
                return;
            }
        }
        self.edit_changed();
        self.sync_audio();
    }

    pub fn set_slider(&mut self, i: usize, v: u8) {
        let before = self.levels();
        self.sliders[i] = v.min(100);
        if i < 5 && self.levels() != before && matches!(self.mode, Mode::Metro | Mode::Loop) {
            self.edit_changed();
        }
        self.sync_audio();
    }

    /// Pad hit from the microphone or the keyboard.
    pub fn pad_hit(&mut self, t: Instant) {
        if !self.power || self.mode != Mode::Coach {
            return;
        }
        self.trig = true;
        self.trig_until = Some(Instant::now() + Duration::from_millis(120));
        if self.playing && self.coach == 0 {
            if let Some(d) = self.audio.beat_offset(t) {
                let tol = [0.06, 0.04, 0.025][(self.guide - 1) as usize];
                let k = ((d.abs() / tol) as u8).min(3);
                // behind the beat (late) shows on the left, ahead (early) on the right
                self.time_check = Some(if d > 0.0 { 3 - k } else { 3 + k });
            }
        }
    }

    // ------------------------------------------------------------ time

    /// Advances timers and pulls playback state from the audio engine.
    pub fn tick(&mut self, now: Instant) {
        let ms = now.duration_since(self.epoch).as_millis();
        self.blink = (ms / BLINK.as_millis()) % 2 == 0;
        self.fast_blink = (ms / 90) % 2 == 0;

        if let Some(t) = self.power_off {
            self.light_level = 0.0;
            self.ink_level = 0.0;
            if now.duration_since(t) >= POWER_OFF_TIME {
                self.power = false;
            }
            return;
        }
        // off, on, off, on, at the blink rate: two flashes in a second, then the value stays lit
        self.flash_off = match self.entry_flash {
            Some(t) if now.duration_since(t) < ENTRY_FLASH => (now.duration_since(t).as_millis() / BLINK.as_millis()) % 2 == 0,
            _ => {
                self.entry_flash = None;
                false
            }
        };
        if self.write_flash_until.is_some_and(|t| now >= t) {
            self.write_flash_until = None;
        }
        if self.trig_until.is_some_and(|t| now >= t) {
            self.trig_until = None;
            self.trig = false;
        }
        if self.coach_name_until.is_some_and(|t| now >= t) {
            self.coach_name_until = None;
            if !self.playing {
                self.show_coach_name = true;
            }
        }
        if self.notice.is_some_and(|(_, t)| now.duration_since(t) > Duration::from_millis(2200)) {
            self.notice = None;
        }

        if !self.playing {
            return;
        }
        let st = self.audio.status();
        if !st.playing {
            self.stop();
            return;
        }
        if self.mode == Mode::Coach {
            self.tempo = st.tempo.round() as u16;
        }
        if self.mode == Mode::Loop && st.loop_memory > 0 {
            if self.memory != st.loop_memory {
                self.memory = st.loop_memory;
                self.load_memory(self.memory);
            }
            self.loop_count = st.loop_count;
        }
        self.leds = [false; 2];
        // the pendulum's position: -1 = left edge, 0 = centre, 1 = right edge
        if let Some(m) = st.last {
            if self.swing_beat != Some(m.pos) {
                if self.swing_beat.is_some() {
                    self.swings += 1;
                }
                self.swing_beat = Some(m.pos);
            }
        }
        let from_left = if self.swings % 2 == 0 { 1.0 } else { -1.0 };
        let now = st.now;
        // at an edge on every beat, straight across to the other edge at an even speed in one
        // beat (the first beat is on the left); in the centre until the first beat sounds
        let pos = match st.last {
            Some(m) => (2.0 * ((now - m.pos as f64) / m.len).clamp(0.0, 1.0) - 1.0) * from_left,
            None => 0.0,
        };
        let i = (((pos + 1.0) / 2.0) * (PEND_N - 1) as f64).round() as usize;
        self.pend = i.min(PEND_N - 1);
        if let Some(m) = st.last {
            if st.now - (m.pos as f64) < 0.08 * st.sr {
                if m.accent || m.num < 0 {
                    self.leds[0] = true;
                } else {
                    self.leds[1] = true;
                }
            }
            if m.num > 0 {
                self.beat_pos = m.num as u8;
            }
        }
    }

    /// True while the LCD shows a field that is being stored (rapid flash).
    pub fn write_flashing(&self) -> bool {
        self.write_flash_until.is_some()
    }

    /// Something moves continuously (the pendulum, the power-off): draw at the display's rate.
    pub fn moving(&self) -> bool {
        self.playing || self.power_off.is_some()
    }

    /// Something time-based changes soon (blinking, flashes): check for changes often.
    pub fn needs_animation(&self) -> bool {
        self.playing || self.notice.is_some() || self.write_flash_until.is_some() || self.power_off.is_some() || self.entry_flash.is_some()
            // a value being edited blinks: keep its on and off phases even
            || self.edit.is_some()
    }
}
