//! Microphone input: onset detection for the Rhythm Coach pad.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::output::hosts;

/// Simple onset detector: block peak crossing a SENS-dependent threshold, with a refractory time.
struct Onset {
    sr: f32,
    level: f32,
    armed: bool,
    last: Option<Instant>,
}

impl Onset {
    fn feed(&mut self, mono: impl Iterator<Item = f32>, n: usize, sens: u32, hits: &Sender<Instant>) {
        // SENS 1..16: higher value = more sensitive (lower threshold)
        let thr = 0.6 * 0.78f32.powi(sens.clamp(1, 16) as i32 - 1);
        let now = Instant::now();
        let mut idx = 0usize;
        let mut block_peak = 0.0f32;
        for x in mono {
            block_peak = block_peak.max(x.abs());
            idx += 1;
            if idx % 64 == 0 || idx == n {
                self.level = self.level * 0.6 + block_peak * 0.4;
                if self.armed && block_peak > thr {
                    let ago = Duration::from_secs_f32((n - idx) as f32 / self.sr);
                    let t = now.checked_sub(ago).unwrap_or(now);
                    if self.last.is_none_or(|l| t.duration_since(l) > Duration::from_millis(90)) {
                        let _ = hits.send(t);
                        self.last = Some(t);
                    }
                    self.armed = false;
                } else if !self.armed && self.level < thr * 0.5 {
                    self.armed = true;
                }
                block_peak = 0.0;
            }
        }
    }
}

pub(super) fn build_mic(sens: Arc<AtomicU32>, hits: Sender<Instant>) -> Result<cpal::Stream, String> {
    let mut last_err = String::from("no audio host");
    for host in hosts() {
        match open_mic(&host, sens.clone(), hits.clone()) {
            Ok(s) => return Ok(s),
            Err(e) => last_err = format!("{:?}: {e}", host.id()),
        }
    }
    Err(last_err)
}

fn open_mic(host: &cpal::Host, sens: Arc<AtomicU32>, hits: Sender<Instant>) -> Result<cpal::Stream, String> {
    let dev = host.default_input_device().ok_or("no input device")?;
    let cfg = dev.default_input_config().map_err(|e| e.to_string())?;
    let ch = cfg.channels() as usize;
    let mut config = cfg.config();
    if let cpal::SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
        config.buffer_size = cpal::BufferSize::Fixed(256u32.clamp(*min, *max));
    }
    let mut det = Onset { sr: cfg.sample_rate() as f32, level: 0.0, armed: true, last: None };
    let err = |e| eprintln!("kadence: mic stream error: {e}");
    let stream = match cfg.sample_format() {
        cpal::SampleFormat::F32 => dev.build_input_stream(
            config,
            move |d: &[f32], _| {
                let n = d.len() / ch;
                det.feed(d.chunks(ch).map(|f| f[0]), n, sens.load(Ordering::Relaxed), &hits);
            },
            err,
            None,
        ),
        cpal::SampleFormat::I32 => dev.build_input_stream(
            config,
            move |d: &[i32], _| {
                let n = d.len() / ch;
                det.feed(d.chunks(ch).map(|f| f[0] as f32 / 2_147_483_648.0), n, sens.load(Ordering::Relaxed), &hits);
            },
            err,
            None,
        ),
        cpal::SampleFormat::I16 => dev.build_input_stream(
            config,
            move |d: &[i16], _| {
                let n = d.len() / ch;
                det.feed(d.chunks(ch).map(|f| f[0] as f32 / 32768.0), n, sens.load(Ordering::Relaxed), &hits);
            },
            err,
            None,
        ),
        f => return Err(format!("unsupported mic sample format {f:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}
