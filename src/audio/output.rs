//! Audio output stream: host selection and the callback that drives the engine.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::engine::Engine;

/// Audio hosts to try, preferred first.
pub(super) fn hosts() -> Vec<cpal::Host> {
    let first = cpal::default_host();
    let first_id = first.id();
    let mut v = vec![first];
    for id in cpal::available_hosts() {
        if id != first_id {
            if let Ok(h) = cpal::host_from_id(id) {
                v.push(h);
            }
        }
    }
    v
}

pub(super) fn open_output(host: &cpal::Host, engine: Arc<Mutex<Engine>>) -> Result<cpal::Stream, String> {
    let dev = host.default_output_device().ok_or("no output device")?;
    let mut cfg = dev.default_output_config().map_err(|e| e.to_string())?;
    if !SUPPORTED.contains(&cfg.sample_format()) {
        // e.g. PipeWire may default to I32 or I24; ask for float at the same rate instead
        if let Some(f) = float_config(&dev, cfg.sample_rate()) {
            cfg = f;
        }
    }
    engine.lock().unwrap_or_else(|p| p.into_inner()).sr = cfg.sample_rate() as f64;
    build_output(&dev, &cfg, engine)
}

/// Requested output buffer: ~10 ms, so START/STOP, tempo and mute react immediately
/// (PulseAudio's default buffer is about two seconds long).
const OUT_FRAMES: u32 = 512;

fn build_output(dev: &cpal::Device, cfg: &cpal::SupportedStreamConfig, engine: Arc<Mutex<Engine>>) -> Result<cpal::Stream, String> {
    let mut config = cfg.config();
    if let cpal::SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
        config.buffer_size = cpal::BufferSize::Fixed(OUT_FRAMES.clamp(*min, *max));
    }
    make_output(dev, cfg.sample_format(), config.clone(), engine.clone()).or_else(|e| {
        if matches!(config.buffer_size, cpal::BufferSize::Fixed(_)) {
            config.buffer_size = cpal::BufferSize::Default;
            make_output(dev, cfg.sample_format(), config, engine)
        } else {
            Err(e)
        }
    })
}

/// Sample formats `make_output` can write.
const SUPPORTED: [cpal::SampleFormat; 4] = [cpal::SampleFormat::F32, cpal::SampleFormat::I32, cpal::SampleFormat::I16, cpal::SampleFormat::U16];

/// A 32-bit float output config at `rate` (or the device's closest), if the device offers one.
fn float_config(dev: &cpal::Device, rate: u32) -> Option<cpal::SupportedStreamConfig> {
    let range = dev.supported_output_configs().ok()?.find(|c| c.sample_format() == cpal::SampleFormat::F32)?;
    let rate = rate.clamp(range.min_sample_rate(), range.max_sample_rate());
    Some(range.with_sample_rate(rate))
}

fn make_output(dev: &cpal::Device, fmt: cpal::SampleFormat, config: cpal::StreamConfig, engine: Arc<Mutex<Engine>>) -> Result<cpal::Stream, String> {
    let stream = match fmt {
        cpal::SampleFormat::F32 => out_stream::<f32>(dev, config, engine, |x| x),
        cpal::SampleFormat::I32 => out_stream::<i32>(dev, config, engine, |x| (x as f64 * i32::MAX as f64) as i32),
        cpal::SampleFormat::I16 => out_stream::<i16>(dev, config, engine, |x| (x * i16::MAX as f32) as i16),
        cpal::SampleFormat::U16 => out_stream::<u16>(dev, config, engine, |x| ((x * 0.5 + 0.5) * u16::MAX as f32) as u16),
        f => return Err(format!("unsupported sample format {f:?}")),
    }?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

fn out_stream<T: cpal::SizedSample + Send + 'static>(
    dev: &cpal::Device,
    config: cpal::StreamConfig,
    engine: Arc<Mutex<Engine>>,
    conv: fn(f32) -> T,
) -> Result<cpal::Stream, String> {
    let channels = config.channels as usize;
    let mut scratch: Vec<f32> = Vec::new();
    dev.build_output_stream(
        config,
        move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
            scratch.resize(data.len(), 0.0);
            let ts = info.timestamp();
            let latency = ts.playback.duration_since(ts.callback).as_secs_f64();
            engine.lock().unwrap_or_else(|p| p.into_inner()).render(&mut scratch, channels, latency);
            for (d, s) in data.iter_mut().zip(&scratch) {
                *d = conv(s.clamp(-1.0, 1.0));
            }
        },
        |e| eprintln!("kadence: audio stream error: {e}"),
        None,
    )
    .map_err(|e| e.to_string())
}
