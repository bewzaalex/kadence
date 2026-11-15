//! Rendering: the panel's parts as SVG layers, rasterized with resvg (gpu.rs puts them on screen).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use resvg::tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use super::cache;
use super::panel::{self as svg, BUTTONS, CAP_REF_Y, DIAL, LCD_RECT, LEDS, SLIDERS};
use crate::device::{Device, Mode, PEND_N};

/// Resolution of the quick background shown while the sharp one is drawn (fraction of full).
const PREVIEW: f32 = 0.25;

/// A picture of one part of the panel, rasterized at `scale`; (x, y) is its top-left pixel and
/// (w, h) its size in pixels.
pub struct Layer {
    /// unique for every picture made, so the GPU knows which textures it already has
    id: u64,
    /// the pixels, until the GPU has them (see `release_pixels`)
    pm: Option<Pixmap>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    scale: f32,
}

/// A layer to draw, and where: (x, y, w, h) in panel units.
pub struct Sprite<'a> {
    pub id: u64,
    /// `None` once the GPU has the pixels
    pub pm: Option<&'a Pixmap>,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Layer {
    /// This layer, moved down by `dy` panel units (rounded to whole pixels, so it stays sharp).
    fn sprite(&self, dy: f64) -> Sprite<'_> {
        let s = self.scale as f64;
        let y = self.y + (dy * s).round() as i32;
        Sprite { id: self.id, pm: self.pm.as_ref(), x: self.x as f64 / s, y: y as f64 / s, w: self.w as f64 / s, h: self.h as f64 / s }
    }

    fn new(pm: Pixmap, x: i32, y: i32, scale: f32) -> Self {
        Layer { id: next_id(), w: pm.width(), h: pm.height(), pm: Some(pm), x, y, scale }
    }
}

pub struct Gfx {
    opt: Arc<usvg::Options<'static>>,
    paint_uri: String,
    scale: f32,
    bg: Option<Layer>,
    /// the sharp background, being drawn on another thread (the old one, or a quick preview,
    /// is shown meanwhile)
    bg_job: Option<Receiver<Layer>>,
    lcd: Option<(String, Layer)>,
    /// each lit pendulum bar, made the first time the needle reaches it, and the bar lit now
    bars: Vec<Option<(String, Layer)>>,
    lit_bar: Option<usize>,
    /// the glass sheen over the LCD
    glass: Option<Layer>,
    knob: Option<(i32, Layer)>,
    caps: Vec<[Layer; 2]>,
    slider_caps: Vec<Layer>,
    leds: Vec<Layer>,
    notice: Option<(&'static str, Layer)>,
}

impl Gfx {
    pub fn new() -> Self {
        let mut opt = usvg::Options { font_family: "Noto Sans".into(), ..Default::default() };
        // the fonts are read in place from the program's data, not copied
        let db = opt.fontdb_mut();
        for font in [&include_bytes!("../../assets/fonts/NotoSans-Bold.ttf")[..], &include_bytes!("../../assets/fonts/NotoSans-Italic.ttf")[..]] {
            db.load_font_source(usvg::fontdb::Source::Binary(Arc::new(font)));
        }
        let paint_uri = format!("data:image/png;base64,{}", base64(include_bytes!("../../assets/paint.png")));
        Gfx {
            opt: Arc::new(opt),
            paint_uri,
            scale: 0.0,
            bg: None,
            bg_job: None,
            lcd: None,
            bars: Vec::new(),
            lit_bar: None,
            glass: None,
            knob: None,
            caps: Vec::new(),
            slider_caps: Vec::new(),
            leds: Vec::new(),
            notice: None,
        }
    }

    /// Rasterizes the part (x, y, w, h) of an SVG document in panel units.
    fn region(&self, doc: &str, x: f64, y: f64, w: f64, h: f64) -> Layer {
        rasterize(&self.opt, doc, self.scale, x, y, w, h)
    }

    /// Puts the background in place: from the disk cache, or else drawn on another thread (see
    /// `poll_background`). Meanwhile the old background stays, drawn at the new size; at the
    /// first start there is none, so a quick low-resolution preview stands in.
    fn start_background(&mut self) {
        let doc = svg::background(&self.paint_uri);
        let (w, h) = self.panel_px();
        let key = cache::key(&doc, w, h);
        self.bg_job = None;
        if let Some(pm) = cache::load(key, w, h) {
            self.bg = Some(Layer::new(pm, 0, 0, self.scale));
            return;
        }
        if self.bg.is_none() {
            self.bg = Some(rasterize(&self.opt, &doc, self.scale * PREVIEW, 0.0, 0.0, svg::W, svg::H));
        }
        let (tx, rx) = mpsc::channel();
        let (opt, scale) = (self.opt.clone(), self.scale);
        std::thread::spawn(move || {
            let l = rasterize(&opt, &doc, scale, 0.0, 0.0, svg::W, svg::H);
            if let Some(pm) = &l.pm {
                cache::save(key, pm);
            }
            let _ = tx.send(l);
        });
        self.bg_job = Some(rx);
    }

    /// Rasterizes many small documents (doc, x, y, w, h) on all CPU cores, in order.
    fn rasterize_all(&self, jobs: &[(String, f64, f64, f64, f64)]) -> Vec<Layer> {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(jobs.len().max(1));
        let per = jobs.len().div_ceil(threads).max(1);
        std::thread::scope(|sc| {
            let handles: Vec<_> = jobs
                .chunks(per)
                .map(|chunk| {
                    let (opt, scale) = (&self.opt, self.scale);
                    sc.spawn(move || chunk.iter().map(|(doc, x, y, w, h)| rasterize(opt, doc, scale, *x, *y, *w, *h)).collect::<Vec<_>>())
                })
                .collect();
            handles.into_iter().flat_map(|h| h.join().expect("render thread")).collect()
        })
    }

    /// Swaps in the sharp background once it is ready; true when the frame must be redrawn.
    pub fn poll_background(&mut self) -> bool {
        let Some(rx) = &self.bg_job else { return false };
        match rx.try_recv() {
            Ok(l) => {
                self.bg = Some(l);
                self.bg_job = None;
                release_free_memory();
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.bg_job = None;
                false
            }
        }
    }

    /// Waits for the sharp background (for snapshots).
    pub fn finish_background(&mut self) {
        if let Some(l) = self.bg_job.take().and_then(|rx| rx.recv().ok()) {
            self.bg = Some(l);
        }
    }

    /// Panel size in pixels at the current scale.
    pub fn panel_px(&self) -> (u32, u32) {
        let s = self.scale as f64;
        ((svg::W * s).ceil() as u32, (svg::H * s).ceil() as u32)
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Rasterizes every part at a new scale (the background on another thread).
    pub fn set_scale(&mut self, scale: f32) {
        let scale = scale.max(0.2);
        if (scale - self.scale).abs() < 1e-4 && self.bg.is_some() {
            return;
        }
        self.scale = scale;
        self.start_background();
        // the small layers: two caps per button, then the slider caps, then the lit LEDs
        let mut jobs = Vec::new();
        for bd in &BUTTONS {
            let (x, y) = (bd.cx - bd.w / 2.0 - 4.0, bd.cy - bd.h / 2.0 - 4.0);
            for pressed in [false, true] {
                jobs.push((svg::button_cap(bd, pressed), x, y, bd.w + 9.0, bd.h + 11.0));
            }
        }
        for (i, x) in SLIDERS.iter().enumerate() {
            jobs.push((svg::slider_cap(i), x - 12.0, CAP_REF_Y - 18.0, 25.0, 38.0));
        }
        for (i, (x, y)) in LEDS.iter().enumerate() {
            jobs.push((svg::led_lit(i), x - 12.0, y - 12.0, 44.0, 32.0));
        }
        let (x, y, w, h) = LCD_RECT;
        jobs.push((super::lcd::glass(), x, y, w, h));
        let mut layers = self.rasterize_all(&jobs).into_iter();
        self.caps = BUTTONS.iter().map(|_| [layers.next().unwrap(), layers.next().unwrap()]).collect();
        self.slider_caps = layers.by_ref().take(SLIDERS.len()).collect();
        self.leds = layers.by_ref().take(LEDS.len()).collect();
        self.glass = layers.next();
        self.lcd = None;
        self.bars = (0..PEND_N).map(|_| None).collect();
        self.knob = None;
        self.notice = None;
        release_free_memory();
    }

    /// Rasterizes the parts that changed with the device state (the LCD, the fan, the knob, the
    /// notice); the others were made by `set_scale`.
    pub fn update(&mut self, d: &Device, knob_angle: f64) {
        let lcd_doc = super::lcd::lcd(d);
        if self.lcd.as_ref().is_none_or(|(k, _)| *k != lcd_doc) {
            let (x, y, w, h) = LCD_RECT;
            let l = self.region(&lcd_doc, x, y, w, h);
            self.lcd = Some((lcd_doc, l));
        }
        // the needle: no lit bar while tuning or with the display off
        self.lit_bar = (d.power && d.mode != Mode::Tune).then_some(d.pend.min(PEND_N - 1));
        if let Some(i) = self.lit_bar {
            let bar_doc = super::lcd::fan_bar(d, i);
            if self.bars[i].as_ref().is_none_or(|(k, _)| *k != bar_doc) {
                let (x, y, w, h) = super::lcd::bar_rect(i);
                let l = self.region(&bar_doc, x, y, w, h);
                self.bars[i] = Some((bar_doc, l));
            }
        }
        let key = (knob_angle * 2.0).round() as i32;
        if self.knob.as_ref().is_none_or(|(k, _)| *k != key) {
            let l = self.region(&svg::knob(key as f64 / 2.0), DIAL.0 - 96.0, DIAL.1 - 96.0, 192.0, 192.0);
            self.knob = Some((key, l));
        }
        match d.notice {
            Some((msg, _)) if self.notice.as_ref().is_none_or(|(m, _)| *m != msg) => {
                let l = self.region(&svg::notice(msg), 0.0, 8.0, svg::W, 44.0);
                self.notice = Some((msg, l));
            }
            None => self.notice = None,
            _ => {}
        }
    }

    /// The panel for the current device state, back to front (after `update`).
    pub fn sprites(&self, d: &Device, pressed: &[crate::device::Btn]) -> Vec<Sprite<'_>> {
        let mut out = Vec::with_capacity(48);
        out.extend(self.bg.as_ref().map(|l| l.sprite(0.0)));
        out.extend(self.lcd.as_ref().map(|(_, l)| l.sprite(0.0)));
        out.extend(self.lit_bar.and_then(|i| self.bars[i].as_ref()).map(|(_, l)| l.sprite(0.0)));
        out.extend(self.glass.as_ref().map(|l| l.sprite(0.0)));
        if d.power {
            out.extend(d.leds.iter().zip(&self.leds).filter(|(on, _)| **on).map(|(_, l)| l.sprite(0.0)));
        }
        for (bd, caps) in BUTTONS.iter().zip(&self.caps) {
            out.push(caps[pressed.contains(&bd.id) as usize].sprite(0.0));
        }
        for (i, l) in self.slider_caps.iter().enumerate() {
            out.push(l.sprite(svg::slider_cap_y(d.sliders[i]) - CAP_REF_Y));
        }
        out.extend(self.knob.as_ref().map(|(_, l)| l.sprite(0.0)));
        out.extend(self.notice.as_ref().map(|(_, l)| l.sprite(0.0)));
        out
    }

    fn layers_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        let one = [self.bg.as_mut(), self.glass.as_mut(), self.lcd.as_mut().map(|l| &mut l.1), self.knob.as_mut().map(|l| &mut l.1), self.notice.as_mut().map(|l| &mut l.1)];
        let bars = self.bars.iter_mut().flatten().map(|b| &mut b.1);
        let many = self.caps.iter_mut().flatten().chain(&mut self.slider_caps).chain(&mut self.leds).chain(bars);
        one.into_iter().flatten().chain(many)
    }

    /// The ids of every layer that exists now.
    pub fn layer_ids(&mut self) -> Vec<u64> {
        self.layers_mut().map(|l| l.id).collect()
    }

    /// Frees the pixels the GPU already holds as textures. Layers not drawn yet (a pressed cap
    /// before its first press) keep theirs until they are.
    pub fn release_pixels(&mut self, uploaded: impl Fn(u64) -> bool) {
        for l in self.layers_mut() {
            if uploaded(l.id) {
                l.pm = None;
            }
        }
    }

    /// The whole panel as one picture, composed on the CPU (for `--render` snapshots).
    pub fn snapshot(&mut self, d: &Device, knob_angle: f64) -> Pixmap {
        self.update(d, knob_angle);
        let (w, h) = self.panel_px();
        let mut pm = Pixmap::new(w, h).expect("pixmap");
        let s = self.scale as f64;
        let paint = PixmapPaint { quality: FilterQuality::Bilinear, ..PixmapPaint::default() };
        for sp in self.sprites(d, &[]) {
            let Some(src) = sp.pm else { continue };
            let k = Transform::from_row((sp.w * s) as f32 / src.width() as f32, 0.0, 0.0, (sp.h * s) as f32 / src.height() as f32, (sp.x * s) as f32, (sp.y * s) as f32);
            pm.draw_pixmap(0, 0, src.as_ref(), &paint, k, None);
        }
        pm
    }
}

fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Rasterizes the part (x, y, w, h) of an SVG document, in panel units, at `scale`.
fn rasterize(opt: &usvg::Options, doc: &str, scale: f32, x: f64, y: f64, w: f64, h: f64) -> Layer {
    let (px0, py0, pw, ph) = pixel_rect(scale, x, y, w, h);
    let mut pm = Pixmap::new(pw, ph).expect("pixmap");
    match usvg::Tree::from_str(doc, opt) {
        Ok(tree) => {
            let t = Transform::from_row(scale, 0.0, 0.0, scale, -px0 as f32, -py0 as f32);
            resvg::render(&tree, t, &mut pm.as_mut());
        }
        Err(e) => eprintln!("kadence: svg error: {e}"),
    }
    Layer::new(pm, px0, py0, scale)
}

/// The pixels a panel area (x, y, w, h) covers at `scale`: whole pixels around it.
fn pixel_rect(scale: f32, x: f64, y: f64, w: f64, h: f64) -> (i32, i32, u32, u32) {
    let s = scale as f64;
    let (px0, py0) = ((x * s).floor() as i32, (y * s).floor() as i32);
    let pw = (((x + w) * s).ceil() as i32 - px0).max(1) as u32;
    let ph = (((y + h) * s).ceil() as i32 - py0).max(1) as u32;
    (px0, py0, pw, ph)
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        o.push(T[(n >> 18) as usize & 63] as char);
        o.push(T[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        o.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    o
}

/// The app icon for the title bar and taskbar on X11 and Windows (Wayland uses the .desktop file).
pub fn window_icon() -> Option<winit::window::Icon> {
    const SIZE: u32 = 64;
    let svg = include_str!("../../assets/icons/io.github.bewzaalex.kadence.svg");
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).ok()?;
    let mut pm = Pixmap::new(SIZE, SIZE)?;
    let k = SIZE as f32 / tree.size().width();
    resvg::render(&tree, Transform::from_scale(k, k), &mut pm.as_mut());
    // winit wants straight (not premultiplied) RGBA
    let rgba = pm.pixels().iter().flat_map(|p| {
        let c = p.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    });
    winit::window::Icon::from_rgba(rgba.collect(), SIZE, SIZE).ok()
}

/// Rasterizing the background allocates large temporary layers; hand that memory back to the OS.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn release_free_memory() {
    unsafe extern "C" {
        fn malloc_trim(pad: usize) -> i32;
    }
    // SAFETY: glibc's malloc_trim only releases free heap pages.
    unsafe {
        malloc_trim(0);
    }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn release_free_memory() {}
