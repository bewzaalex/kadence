//! What both spike variants share: the window, the scene and the measurements.
//!
//! The scene is what Kadence draws: the panel (a picture rasterized once, here a 2x PNG made with
//! `kadence --render`), and over it the moving parts, drawn by the GPU every frame: a swinging
//! needle and 27 fan bars on the LCD, and a turning marker on the dial.

use std::f32::consts::PI;
use std::time::{Duration, Instant};

use winit::dpi::LogicalSize;
use winit::window::{Window, WindowAttributes};

/// Panel size in panel units (as in src/ui/panel.rs).
pub const W: f32 = 610.0;
pub const H: f32 = 800.0;

/// The panel picture: premultiplied RGBA.
pub struct Picture {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

pub fn load_panel() -> Picture {
    let path = std::env::args().nth(1).unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/panel.png").into());
    let pm = tiny_skia::Pixmap::load_png(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Picture { w: pm.width(), h: pm.height(), rgba: pm.take() }
}

/// The picture and its smaller versions, each half the size of the one before (mipmaps), so the
/// GPU can show it smoothly at any smaller size. Made on the CPU, the same for both variants.
pub fn mip_levels(p: Picture) -> Vec<Picture> {
    let mut levels = vec![p];
    loop {
        let a = levels.last().unwrap();
        if a.w == 1 && a.h == 1 {
            return levels;
        }
        let (w, h) = ((a.w / 2).max(1), (a.h / 2).max(1));
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                for c in 0..4 {
                    let at = |dx: u32, dy: u32| {
                        let (sx, sy) = ((x * 2 + dx).min(a.w - 1), (y * 2 + dy).min(a.h - 1));
                        a.rgba[((sy * a.w + sx) * 4 + c) as usize] as u32
                    };
                    rgba[((y * w + x) * 4 + c) as usize] = ((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1) + 2) / 4) as u8;
                }
            }
        }
        levels.push(Picture { w, h, rgba });
    }
}

pub fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_title("GPU spike")
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size(LogicalSize::new(W * 0.8, H * 0.8))
}

/// One vertex: position in clip space, texture position, premultiplied colour, and 1 for a plain
/// colour (no texture).
pub const VERTEX_FLOATS: usize = 9;

/// The frame's triangles, for a window of any size (the panel fills it).
pub fn scene(t: f32) -> Vec<f32> {
    let mut v = Vec::with_capacity(32 * 6 * VERTEX_FLOATS);
    // panel units → clip space
    let p = |x: f32, y: f32| [x / W * 2.0 - 1.0, 1.0 - y / H * 2.0];
    let mut quad = |c: [[f32; 2]; 4], uv: [[f32; 2]; 4], rgba: [f32; 4], solid: f32| {
        for i in [0, 1, 2, 0, 2, 3] {
            let q = p(c[i][0], c[i][1]);
            v.extend_from_slice(&[q[0], q[1], uv[i][0], uv[i][1], rgba[0], rgba[1], rgba[2], rgba[3], solid]);
        }
    };
    let none = [[0.0; 2]; 4];
    quad([[0.0, 0.0], [W, 0.0], [W, H], [0.0, H]], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], [1.0; 4], 0.0);

    // the needle swings linearly at 120 BPM (one beat per side), like Kadence's pendulum
    let phase = (t * 2.0).rem_euclid(2.0);
    let swing = if phase < 1.0 { phase * 2.0 - 1.0 } else { 3.0 - phase * 2.0 };
    let angle = swing * 0.62;
    // 27 fan bars across the LCD's upper half; the bar under the needle is dark
    let (fx, fy, fw, fh) = (78.0, 236.0, 214.0, 100.0);
    let lit = ((swing + 1.0) / 2.0 * 26.0).round() as i32;
    for i in 0..27 {
        let x = fx + fw * i as f32 / 27.0;
        let a = if i == lit { 0.85 } else { 0.07 };
        let h = fh * (0.55 + 0.45 * (1.0 - ((i as f32 - 13.0) / 13.0).powi(2)));
        quad([[x, fy + fh - h], [x + 5.0, fy + fh - h], [x + 5.0, fy + fh], [x, fy + fh]], none, [0.1 * a, 0.06 * a, 0.02 * a, a], 1.0);
    }
    rotated(&mut quad, (185.0, 410.0), angle, 1.6, 150.0, [0.1, 0.06, 0.02, 0.9]);
    // a marker turning on the dial
    rotated(&mut quad, (492.0, 524.0), t * PI / 2.0, 3.0, 60.0, [0.0, 0.0, 0.0, 0.5]);
    v
}

/// A bar of half-width `hw` and length `len` from `pivot`, turned by `angle` from straight up.
fn rotated(quad: &mut impl FnMut([[f32; 2]; 4], [[f32; 2]; 4], [f32; 4], f32), pivot: (f32, f32), angle: f32, hw: f32, len: f32, rgba: [f32; 4]) {
    let (s, c) = angle.sin_cos();
    let at = |x: f32, y: f32| [pivot.0 + x * c - y * s, pivot.1 + x * s + y * c];
    quad([at(-hw, -len), at(hw, -len), at(hw, 0.0), at(-hw, 0.0)], [[0.0; 2]; 4], rgba, 1.0);
}

/// Measurements, printed every 2 s and as a summary when the spike ends.
pub struct Stats {
    name: &'static str,
    start: Instant,
    pub first_frame: Option<Duration>,
    frames: u32,
    window: (Instant, u32, f64),
    peak_rss: f64,
    pub info: String,
}

impl Stats {
    pub fn new(name: &'static str) -> Self {
        let now = Instant::now();
        Stats { name, start: now, first_frame: None, frames: 0, window: (now, 0, cpu_seconds()), peak_rss: 0.0, info: String::new() }
    }

    /// Seconds since start, for the animation.
    pub fn t(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Call after every presented frame; false when the run should end (SPIKE_SECONDS).
    pub fn frame(&mut self) -> bool {
        self.frames += 1;
        if self.first_frame.is_none() {
            self.first_frame = Some(self.start.elapsed());
            eprintln!("{}: first frame after {:?} ({})", self.name, self.start.elapsed(), self.info);
        }
        let (t0, f0, c0) = self.window;
        let dt = t0.elapsed().as_secs_f64();
        if dt >= 2.0 {
            let rss = rss_mb();
            self.peak_rss = self.peak_rss.max(rss);
            let cpu = cpu_seconds();
            eprintln!(
                "{}: {:.0} fps, CPU {:.1}%, RSS {:.1} MB",
                self.name,
                (self.frames - f0) as f64 / dt,
                (cpu - c0) / dt * 100.0,
                rss
            );
            self.window = (Instant::now(), self.frames, cpu);
        }
        let limit: f32 = std::env::var("SPIKE_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(f32::INFINITY);
        if self.t() < limit {
            return true;
        }
        let secs = self.start.elapsed().as_secs_f64();
        eprintln!(
            "SUMMARY {}: first frame {:.0} ms, {:.0} fps average, CPU {:.1}% average, RSS now {:.1} MB, peak {:.1} MB",
            self.name,
            self.first_frame.unwrap().as_secs_f64() * 1000.0,
            self.frames as f64 / secs,
            cpu_seconds() / secs * 100.0,
            rss_mb(),
            self.peak_rss.max(rss_mb())
        );
        false
    }
}

/// SPIKE_RESIZE=1: the window grows and shrinks all the time, to show how resizing looks.
pub fn resize_test(window: &Window, t: f32) {
    if std::env::var_os("SPIKE_RESIZE").is_some() {
        let k = 0.6 + 0.3 * (t * 1.5).sin();
        let _ = window.request_inner_size(LogicalSize::new(W * k, H * k));
    }
}

fn proc_field(file: &str, i: usize) -> f64 {
    let s = std::fs::read_to_string(file).unwrap_or_default();
    // /proc/self/stat: skip past the command name, which may contain spaces
    let s = s.rsplit_once(')').map_or(s.as_str(), |(_, r)| r);
    s.split_whitespace().nth(i).and_then(|x| x.parse().ok()).unwrap_or(0.0)
}

/// Process CPU time (user + system), in seconds.
fn cpu_seconds() -> f64 {
    // fields 14 and 15 of /proc/self/stat; after the ')' they are the 12th and 13th
    (proc_field("/proc/self/stat", 11) + proc_field("/proc/self/stat", 12)) / 100.0
}

/// Resident memory, in MB.
pub fn rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
    let pages: f64 = s.split_whitespace().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    pages * 4096.0 / 1_048_576.0
}

/// The shader both variants use, in WGSL for wgpu (the GL variant has the same in GLSL).
pub const WGSL: &str = r#"
struct V { @builtin(position) pos: vec4f, @location(0) uv: vec2f, @location(1) rgba: vec4f, @location(2) solid: f32 }
@vertex fn vs(@location(0) p: vec2f, @location(1) uv: vec2f, @location(2) rgba: vec4f, @location(3) solid: f32) -> V {
    return V(vec4f(p, 0.0, 1.0), uv, rgba, solid);
}
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@fragment fn fs(v: V) -> @location(0) vec4f {
    let t = textureSample(tex, smp, v.uv);
    return select(t * v.rgba, v.rgba, v.solid > 0.5);
}
"#;
