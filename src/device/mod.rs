//! KD-90 behaviour: modes, buttons, dial, memories and the LCD state (port of the mockup logic).

mod store;

use std::time::{Duration, Instant};

use crate::audio::{Audio, LoopStep, Params, PlayMode};

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
    Tempo,
    Memory,
    Write,
    Beat,
    Beat2,
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

pub const BLINK: Duration = Duration::from_millis(500);
const EDIT_TIMEOUT: Duration = Duration::from_secs(6);
/// POWER: the backlight fades, then the LCD segments fade, then the app quits.
const LIGHT_FADE: Duration = Duration::from_millis(350);
const INK_FADE: Duration = Duration::from_millis(450);
const POWER_OFF_TIME: Duration = Duration::from_millis(900);

pub struct Device {
    pub power: bool,
    pub backlight: bool,
    /// set when POWER was pressed; the panel animates off and the app quits
    power_off: Option<Instant>,
    /// LCD backlight and segment strength during the power-off animation (1 = normal)
    pub light_level: f64,
    pub ink_level: f64,
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
    pub pend: usize,
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
    last_edit: Instant,
    edit_until: Option<Instant>,
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
            pend: 8,
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
            last_edit: now,
            edit_until: None,
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
        d.audio.sens.store(d.sens as u32, std::sync::atomic::Ordering::Relaxed);
        if d.audio.error.is_some() {
            d.notice("No audio output device: sound is off");
        }
        d.sync_audio();
        d
    }

    // ------------------------------------------------------------ helpers

    pub fn levels(&self) -> [u8; 5] {
        let mut l = [0; 5];
        for i in 0..5 {
            l[i] = ((self.sliders[i] as f32) / 20.0).round() as u8;
        }
        l
    }

    fn params(&self) -> Params {
        let lv = self.levels();
        Params {
            tempo: self.tempo as f64,
            pattern: self.pattern_mode.then(|| self.pattern - 1),
            beat1: self.beat1,
            beat2: self.beat2,
            voice: self.voice,
            levels: lv.map(|l| l as f32 / 5.0),
            muted: self.muted,
            mode: match self.mode {
                Mode::Coach => PlayMode::Coach(self.coach),
                Mode::Loop => PlayMode::Loop,
                _ => PlayMode::Metro,
            },
            count_in: self.count_in,
        }
    }

    fn mem_params(&self, m: &Mem) -> Params {
        Params {
            tempo: m.tempo as f64,
            pattern: m.pattern_mode.then(|| m.pattern - 1),
            beat1: m.beat1,
            beat2: m.beat2,
            voice: m.voice,
            levels: m.sliders.map(|v| (v as f32 / 20.0).round() / 5.0),
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
        self.tempo = m.tempo.clamp(30, 250);
        self.pattern_mode = m.pattern_mode;
        self.beat1 = m.beat1.clamp(1, 9);
        self.beat2 = m.beat2.min(9);
        self.pattern = m.pattern.clamp(1, 30);
        self.voice = m.voice.clamp(1, 4);
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

    fn touch(&mut self, now: Instant) {
        self.last_edit = now;
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
        self.edit = None;
        self.show_coach_name = false;
        if self.mode == Mode::Coach {
            self.audio_ensure_mic();
        }
    }

    fn stop(&mut self) {
        self.audio.stop();
        self.playing = false;
        self.pend = 8;
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

    fn audio_ensure_mic(&mut self) {
        self.audio.ensure_mic();
    }

    fn set_mode(&mut self, m: Mode) {
        if self.playing {
            self.stop();
        }
        if self.mode == Mode::Tune && m != Mode::Tune {
            self.set_tone(false);
        }
        self.mode = m;
        self.edit = None;
        if m == Mode::Coach {
            self.audio_ensure_mic();
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
        self.touch(now);
        if b != Btn::Light && self.edit == Some(Edit::Tempo) {
            self.edit = None;
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
                    self.edit = if self.edit == Some(Edit::Memory) { None } else { Some(Edit::Memory) };
                }
            }
            Btn::Write => self.write(now),
            Btn::Midi => self.notice("MIDI input is not implemented yet"),
            Btn::Tempo => {
                if self.mode != Mode::Metro {
                    self.set_mode(Mode::Metro);
                }
                // the tempo value flashes a number of times
                self.edit = Some(Edit::Tempo);
                self.edit_until = Some(now + Duration::from_millis(1600));
            }
            Btn::Loop => {
                self.set_mode(Mode::Loop);
                self.loop_repeats = self.memories[self.memory as usize - 1].repeats;
                self.loop_count = 0;
            }
            Btn::Tune => {
                if self.mode == Mode::Tune {
                    self.edit = None;
                } else {
                    self.set_mode(Mode::Tune);
                    let t = self.tones[self.tone_memory as usize - 1];
                    self.note = t.note;
                    self.pitch = t.pitch;
                }
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
                    self.edit = match self.edit {
                        None => Some(Edit::Sens),
                        Some(Edit::Sens) => Some(Edit::Guide),
                        _ => None,
                    };
                }
            }
            Btn::Beat | Btn::Voice if combo => {
                // [BEAT/PATTERN] + [VOICE]: count-in on/off
                if self.count_in > 0 {
                    self.count_in = 0;
                    self.edit = None;
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
                self.edit = if self.edit == Some(Edit::Beat) && !self.pattern_mode { Some(Edit::Beat2) } else { Some(Edit::Beat) };
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
                if self.edit == Some(Edit::Write) {
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

    /// POWER turns the app off: playback stops, the display fades out and the app quits.
    /// Holding MEMORY while pressing POWER restores the factory settings instead.
    fn power_button(&mut self, now: Instant) {
        if self.is_held(Btn::Memory) {
            self.factory_reset();
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
        self.sens = 8;
        self.guide = 2;
        self.sliders = [100, 100, 0, 0, 0, self.sliders[5]];
        self.load_memory(1);
        self.audio.sens.store(self.sens as u32, std::sync::atomic::Ordering::Relaxed);
        self.sync_audio();
        self.save();
        self.notice("Factory settings restored");
    }

    fn write(&mut self, now: Instant) {
        if self.mode == Mode::Coach && self.playing && self.coach >= 2 {
            // tempo in effect becomes the maximum metronome value
            self.audio.coach_set_max();
            return;
        }
        if self.edit == Some(Edit::Write) {
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
            self.tempo = ((60.0 / avg).round() as u16).clamp(30, 250);
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
        self.touch(now);
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
            Some(Edit::Beat) => {
                // the dial runs BEAT 1..9, then PATTERN 1..30
                let idx = if self.pattern_mode { 9 + self.pattern as i32 } else { self.beat1 as i32 };
                let idx = add(idx, 1, 39);
                if idx <= 9 {
                    self.pattern_mode = false;
                    self.beat1 = idx as u8;
                } else {
                    self.pattern_mode = true;
                    self.pattern = (idx - 9) as u8;
                }
            }
            Some(Edit::Beat2) => self.beat2 = add(self.beat2 as i32, 0, 9) as u8,
            Some(Edit::Voice) => self.voice = add(self.voice as i32, 1, 4) as u8,
            Some(Edit::CountIn) => {
                self.count_in = add(self.count_in as i32, 1, 8) as u8;
                self.sync_audio();
                return;
            }
            Some(Edit::Sens) => {
                self.sens = add(self.sens as i32, 1, 16) as u8;
                self.audio.sens.store(self.sens as u32, std::sync::atomic::Ordering::Relaxed);
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
            Some(Edit::Tempo) | None => {
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
                        self.tempo = add(self.tempo as i32, 30, 250) as u16;
                        self.show_coach_name = false;
                        self.coach_name_until = Some(now + Duration::from_millis(1500));
                        self.sync_audio();
                    }
                    Mode::Metro => {
                        self.tempo = add(self.tempo as i32, 30, 250) as u16;
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
            let t = now.duration_since(t);
            let ease = |x: f64| 1.0 - x.clamp(0.0, 1.0).powi(2);
            self.light_level = ease(t.as_secs_f64() / LIGHT_FADE.as_secs_f64());
            self.ink_level = ease((t.as_secs_f64() - LIGHT_FADE.as_secs_f64() * 0.6) / INK_FADE.as_secs_f64());
            if t >= POWER_OFF_TIME {
                self.power = false;
            }
            return;
        }
        if self.edit_until.is_some_and(|t| now >= t) {
            self.edit_until = None;
            if self.edit == Some(Edit::Tempo) {
                self.edit = None;
            }
        }
        if self.edit.is_some_and(|e| !matches!(e, Edit::Write | Edit::CountIn | Edit::Tempo))
            && now.duration_since(self.last_edit) > EDIT_TIMEOUT
        {
            self.edit = None;
        }
        if self.edit == Some(Edit::CountIn) && now.duration_since(self.last_edit) > EDIT_TIMEOUT {
            self.edit = None;
        }
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
        if let Some(m) = st.last {
            let ph = ((st.now - m.pos as f64) / m.len).clamp(0.0, 1.0);
            // pendulum: at an extreme on the beat, swings across in one beat
            let dir = if m.num.rem_euclid(2) == 1 { 1.0 } else { -1.0 };
            let pos = (std::f64::consts::PI * ph).cos() * dir;
            self.pend = (((pos + 1.0) / 2.0) * 16.0).round() as usize;
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

    /// Next instant something time-based changes, for the event loop.
    pub fn needs_animation(&self) -> bool {
        self.playing || self.notice.is_some() || self.write_flash_until.is_some() || self.power_off.is_some()
    }
}
