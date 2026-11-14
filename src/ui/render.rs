//! Rendering: SVG layers rasterized with resvg, composited into one frame.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use std::hash::{DefaultHasher, Hash, Hasher};

use resvg::tiny_skia::{FilterQuality, IntRect, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use super::cache;
use super::panel::{self as svg, BUTTONS, CAP_REF_Y, DIAL, LCD_RECT, LEDS, SLIDERS};
use crate::device::Device;

/// Resolution of the quick background shown while the sharp one is drawn (fraction of full).
const PREVIEW: f32 = 0.25;

/// A rectangle of the frame, in pixels.
#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// A composed frame, and the part that changed since the last one (`None`: all of it).
pub struct Frame<'a> {
    pub canvas: &'a Pixmap,
    pub changed: Option<Area>,
}

struct Layer {
    pm: Pixmap,
    x: i32,
    y: i32,
}

pub struct Gfx {
    opt: Arc<usvg::Options<'static>>,
    paint_uri: String,
    scale: f32,
    bg: Option<Pixmap>,
    /// counts background changes (preview, sharp version, new size), so the composed `still`
    /// picture is rebuilt whenever the background under it changes
    bg_gen: u64,
    /// the sharp background, being drawn on another thread (the preview is shown meanwhile)
    bg_job: Option<Receiver<Pixmap>>,
    canvas: Option<Pixmap>,
    lcd: Option<(String, Layer)>,
    /// the pendulum fan over the LCD, and the glass sheen over both
    fan: Option<(String, Layer)>,
    glass: Option<Layer>,
    /// the glass over the fan's area, laid over the fan on every frame
    glass_fan: Option<Pixmap>,
    /// everything but the fan, composed (redone only when one of its parts changes)
    still: Option<(u64, Pixmap)>,
    knob: Option<(i32, Layer)>,
    caps: Vec<[Layer; 2]>,
    slider_caps: Vec<Layer>,
    leds: Vec<Layer>,
    notice: Option<(&'static str, Layer)>,
}

impl Gfx {
    pub fn new() -> Self {
        let mut opt = usvg::Options::default();
        opt.font_family = "Noto Sans".into();
        let db = opt.fontdb_mut();
        db.load_font_data(include_bytes!("../../assets/fonts/NotoSans-Bold.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/NotoSans-Italic.ttf").to_vec());
        let paint_uri = format!("data:image/png;base64,{}", base64(include_bytes!("../../assets/paint.png")));
        Gfx {
            opt: Arc::new(opt),
            paint_uri,
            scale: 0.0,
            bg: None,
            bg_gen: 0,
            bg_job: None,
            canvas: None,
            lcd: None,
            fan: None,
            glass: None,
            glass_fan: None,
            still: None,
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

    /// Puts the background in place: from the disk cache, or else a quick low-resolution
    /// preview now and the sharp version from another thread (see `poll_background`).
    fn start_background(&mut self) {
        let doc = svg::background(&self.paint_uri);
        let (w, h) = self.panel_px();
        let key = cache::key(&doc, w, h);
        self.bg_job = None;
        if let Some(pm) = cache::load(key, w, h) {
            self.set_bg(pm);
            return;
        }
        let small = rasterize(&self.opt, &doc, self.scale * PREVIEW, 0.0, 0.0, svg::W, svg::H).pm;
        let mut preview = Pixmap::new(w, h).expect("pixmap");
        let paint = PixmapPaint { quality: FilterQuality::Bilinear, ..PixmapPaint::default() };
        let k = Transform::from_scale(w as f32 / small.width() as f32, h as f32 / small.height() as f32);
        preview.draw_pixmap(0, 0, small.as_ref(), &paint, k, None);
        self.set_bg(preview);

        let (tx, rx) = mpsc::channel();
        let (opt, scale) = (self.opt.clone(), self.scale);
        std::thread::spawn(move || {
            let pm = rasterize(&opt, &doc, scale, 0.0, 0.0, svg::W, svg::H).pm;
            cache::save(key, &pm);
            let _ = tx.send(pm);
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
            Ok(pm) => {
                self.set_bg(pm);
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
        if let Some(pm) = self.bg_job.take().and_then(|rx| rx.recv().ok()) {
            self.set_bg(pm);
        }
    }

    fn set_bg(&mut self, pm: Pixmap) {
        self.bg = Some(pm);
        self.bg_gen += 1;
    }

    /// Panel size in pixels at the current scale.
    pub fn panel_px(&self) -> (u32, u32) {
        let s = self.scale as f64;
        ((svg::W * s).ceil() as u32, (svg::H * s).ceil() as u32)
    }

    pub fn set_scale(&mut self, scale: f32) {
        let scale = scale.max(0.2);
        if (scale - self.scale).abs() < 1e-4 && self.bg.is_some() {
            return;
        }
        self.scale = scale;
        // drop the old frame buffers first so peak memory stays low while resizing
        self.bg = None;
        self.canvas = None;
        self.start_background();
        let (w, h) = self.panel_px();
        self.canvas = Pixmap::new(w, h);
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
        let mut layers = self.rasterize_all(&jobs).into_iter();
        self.caps = BUTTONS.iter().map(|_| [layers.next().unwrap(), layers.next().unwrap()]).collect();
        self.slider_caps = layers.by_ref().take(SLIDERS.len()).collect();
        self.leds = layers.collect();
        self.lcd = None;
        self.fan = None;
        self.still = None;
        let (x, y, w, h) = LCD_RECT;
        let glass = self.region(&super::lcd::glass(), x, y, w, h);
        // the same sheen, cut to the fan's area (the fan layer is placed on the same pixel grid)
        let (fx, fy, fw, fh) = super::lcd::fan_rect();
        let (ax, ay, aw, ah) = pixel_rect(self.scale, fx, fy, fw, fh);
        let cut = IntRect::from_xywh(ax - glass.x, ay - glass.y, aw, ah);
        self.glass_fan = cut.and_then(|r| glass.pm.clone_rect(r));
        self.glass = Some(glass);
        self.knob = None;
        self.notice = None;
        release_free_memory();
    }

    /// Composes the whole panel for the current device state.
    pub fn compose(&mut self, d: &Device, knob_angle: f64, pressed: &[crate::device::Btn]) -> Frame<'_> {
        let lcd_doc = super::lcd::lcd(d);
        if self.lcd.as_ref().is_none_or(|(k, _)| *k != lcd_doc) {
            let (x, y, w, h) = LCD_RECT;
            let l = self.region(&lcd_doc, x, y, w, h);
            self.lcd = Some((lcd_doc, l));
        }
        let fan_doc = super::lcd::fan(d);
        if self.fan.as_ref().is_none_or(|(k, _)| *k != fan_doc) {
            let (x, y, w, h) = super::lcd::fan_rect();
            let l = self.region(&fan_doc, x, y, w, h);
            self.fan = Some((fan_doc, l));
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

        // everything but the fan is composed into `still`, and redone only when it changes
        let s = self.scale as f64;
        let mut key = DefaultHasher::new();
        (self.bg_gen, &self.lcd.as_ref().map(|l| &l.0), self.knob.as_ref().map(|k| k.0), pressed, d.sliders, d.power && d.leds[0], d.power && d.leds[1]).hash(&mut key);
        d.notice.map(|n| n.0).hash(&mut key);
        let key = key.finish();
        let paint = PixmapPaint::default();
        let full = self.still.as_ref().is_none_or(|(k, _)| *k != key);
        if full {
            let mut still = self.bg.clone().expect("scale set");
            let mut blit = |l: &Layer, dy: i32| still.draw_pixmap(l.x, l.y + dy, l.pm.as_ref(), &paint, Transform::identity(), None);
            if let Some((_, l)) = &self.lcd {
                blit(l, 0);
            }
            if let Some(l) = &self.glass {
                blit(l, 0);
            }
            if d.power {
                for (i, on) in d.leds.iter().enumerate() {
                    if *on {
                        blit(&self.leds[i], 0);
                    }
                }
            }
            for (i, bd) in BUTTONS.iter().enumerate() {
                let down = pressed.contains(&bd.id);
                blit(&self.caps[i][down as usize], 0);
            }
            for i in 0..6 {
                let dy = ((svg::slider_cap_y(d.sliders[i]) - CAP_REF_Y) * s).round() as i32;
                blit(&self.slider_caps[i], dy);
            }
            if let Some((_, l)) = &self.knob {
                blit(l, 0);
            }
            if let Some((_, l)) = &self.notice {
                blit(l, 0);
            }
            self.still = Some((key, still));
        }

        // the frame: `still`, and over it the fan with the glass on top, in the fan's area only
        let still = &self.still.as_ref().unwrap().1;
        let canvas = self.canvas.as_mut().expect("scale set");
        let (fan, glass) = (&self.fan.as_ref().expect("fan drawn").1, self.glass_fan.as_ref().expect("scale set"));
        let area = Area { x: fan.x, y: fan.y, w: fan.pm.width(), h: fan.pm.height() };
        if full {
            canvas.data_mut().copy_from_slice(still.data());
        } else {
            let row = canvas.width() as usize * 4;
            let (x0, x1) = (area.x.max(0) as usize * 4, ((area.x + area.w as i32) as usize).min(canvas.width() as usize) * 4);
            for y in area.y.max(0) as usize..((area.y + area.h as i32) as usize).min(canvas.height() as usize) {
                canvas.data_mut()[y * row + x0..y * row + x1].copy_from_slice(&still.data()[y * row + x0..y * row + x1]);
            }
        }
        canvas.draw_pixmap(fan.x, fan.y, fan.pm.as_ref(), &paint, Transform::identity(), None);
        canvas.draw_pixmap(fan.x, fan.y, glass.as_ref(), &paint, Transform::identity(), None);
        Frame { canvas: self.canvas.as_ref().unwrap(), changed: if full { None } else { Some(area) } }
    }
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
    Layer { pm, x: px0, y: py0 }
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
