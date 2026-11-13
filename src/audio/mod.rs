//! Sound: sample-accurate sequencer, synthesized voices and drums, reference tone,
//! and microphone onset detection for the Rhythm Coach.

mod engine;
mod mic;
mod output;
pub mod patterns;
mod synth;

use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub use engine::{LoopStep, Params, PlayMode, Status};

use engine::Engine;
use mic::build_mic;
use output::{hosts, open_output};

pub struct Audio {
    engine: Arc<Mutex<Engine>>,
    _out: Option<cpal::Stream>,
    mic: Option<cpal::Stream>,
    mic_tried: bool,
    pub sens: Arc<AtomicU32>,
    hits: Sender<Instant>,
    pub error: Option<String>,
}

impl Audio {
    pub fn new(hits: Sender<Instant>) -> Self {
        let engine = Arc::new(Mutex::new(Engine::new(48_000.0)));
        let mut errors = Vec::new();
        let mut stream = None;
        // cpal's default host is PulseAudio (also served by PipeWire) when a server is running,
        // otherwise ALSA; if the preferred host fails we still try the others.
        for host in hosts() {
            match open_output(&host, engine.clone()) {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(e) => errors.push(format!("{:?}: {e}", host.id())),
            }
        }
        let error = (stream.is_none()).then(|| if errors.is_empty() { "no audio host".into() } else { errors.join("; ") });
        if let Some(e) = &error {
            eprintln!("kadence: no audio output ({e})");
        }
        Audio { engine, _out: stream, mic: None, mic_tried: false, sens: Arc::new(AtomicU32::new(8)), hits, error }
    }

    fn with<R>(&self, f: impl FnOnce(&mut Engine) -> R) -> R {
        let mut e = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        f(&mut e)
    }

    pub fn start(&self, params: Params, program: Vec<LoopStep>) {
        self.with(|e| e.start(params, program));
    }
    pub fn stop(&self) {
        self.with(|e| e.playing = false);
    }
    pub fn update(&self, params: Params) {
        self.with(|e| e.update(params));
    }
    pub fn set_master(&self, vol: f32) {
        self.with(|e| e.master = 0.85 * vol * vol);
    }
    /// Sets the Coach maximum tempo to the current tempo (WRITE during Gradual/Step).
    pub fn coach_set_max(&self) {
        self.with(|e| {
            e.coach_max = e.tempo;
            e.coach_dir = -1.0;
        });
    }
    pub fn tone(&self, freq: Option<f32>) {
        self.with(|e| e.tone_freq = freq);
    }
    pub fn status(&self) -> Status {
        self.with(|e| e.status())
    }
    pub fn beat_offset(&self, t: Instant) -> Option<f64> {
        self.with(|e| e.beat_offset(t))
    }

    /// Opens the default microphone the first time Coach mode needs it.
    pub fn ensure_mic(&mut self) {
        if self.mic_tried {
            return;
        }
        self.mic_tried = true;
        match build_mic(self.sens.clone(), self.hits.clone()) {
            Ok(s) => self.mic = Some(s),
            Err(e) => eprintln!("kadence: microphone unavailable ({e}); use the K key as the pad"),
        }
    }
}
