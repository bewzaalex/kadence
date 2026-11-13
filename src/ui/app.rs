//! The window: draws the panel and turns mouse, wheel and keyboard input into panel actions.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use super::panel::{self as svg, BUTTONS, DIAL, SLIDERS, SL_BOT, SL_TOP};
use super::render::Gfx;
use crate::device::{Btn, Device};

const BG: u32 = 0x1d1e20;
/// Dial degrees per value step (detent).
const DETENT: f64 = 12.0;
/// Near the dial's centre the hand's angle is too noisy to tell the direction; ignore it there.
const DIAL_DEAD_RADIUS: f64 = 28.0;

struct Surface {
    window: Rc<Window>,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
}

enum Drag {
    Slider(usize),
    Dial(f64),
}

pub struct App {
    dev: Device,
    gfx: Gfx,
    hits: Receiver<Instant>,
    win: Option<Surface>,
    size: (u32, u32),
    offset: (i32, i32),
    cursor: PhysicalPosition<f64>,
    mouse_btn: Option<Btn>,
    drag: Option<Drag>,
    knob_angle: f64,
    dial_acc: f64,
    wheel_acc: f64,
    last_sig: String,
    /// window frame on (`--decorations`); otherwise the case is the window
    decorations: bool,
    /// borderless window with see-through corners (needs alpha support: Linux, macOS)
    transparent: bool,
    /// window size relative to the panel's natural size
    zoom: f64,
    ctrl: bool,
}

impl App {
    /// `transparent`: borderless window with see-through corners (needs alpha support).
    pub fn new(dev: Device, hits: Receiver<Instant>, decorations: bool, transparent: bool) -> Self {
        App {
            dev,
            gfx: Gfx::new(),
            hits,
            win: None,
            size: (1, 1),
            offset: (0, 0),
            cursor: PhysicalPosition::new(0.0, 0.0),
            mouse_btn: None,
            drag: None,
            knob_angle: 0.0,
            dial_acc: 0.0,
            wheel_acc: 0.0,
            last_sig: String::new(),
            decorations,
            transparent,
            zoom: 1.0,
            ctrl: false,
        }
    }

    fn to_panel(&self, p: PhysicalPosition<f64>) -> (f64, f64) {
        let s = self.gfx_scale();
        ((p.x - self.offset.0 as f64) / s, (p.y - self.offset.1 as f64) / s)
    }

    fn gfx_scale(&self) -> f64 {
        let (w, h) = self.size;
        (w as f64 / svg::W).min(h as f64 / svg::H)
    }

    /// The dial is an encoder: it rests only on detents and clicks one step at a time.
    /// `dial_acc` is how far the hand has moved since the last click; a click needs a full
    /// detent of travel in one direction, so small jitter never steps back and forth.
    fn turn(&mut self, deg: f64) {
        self.dial_acc += deg;
        let steps = (self.dial_acc / DETENT).trunc() as i32;
        self.dial_acc -= steps as f64 * DETENT;
        if steps != 0 {
            self.knob_angle = (self.knob_angle + steps as f64 * DETENT).rem_euclid(360.0);
            self.dev.dial(steps, Instant::now());
        }
    }

    /// Keeps our idea of the window size in step with the real one. On Wayland the size can
    /// change with the scale factor without a Resized event reaching us first.
    fn sync_size(&mut self) {
        if let Some(sf) = &self.win {
            let s = sf.window.inner_size();
            if (s.width.max(1), s.height.max(1)) != self.size {
                self.resize(s.width, s.height);
            }
        }
    }

    fn mouse_down(&mut self) {
        self.sync_size();
        let (x, y) = self.to_panel(self.cursor);
        debug(format_args!("click {:?} -> panel ({x:.1},{y:.1}) size {:?} offset {:?} scale {:.3}", self.cursor, self.size, self.offset, self.gfx_scale()));
        let now = Instant::now();
        for bd in &BUTTONS {
            if (x - bd.cx).abs() <= bd.w / 2.0 && (y - bd.cy).abs() <= bd.h / 2.0 {
                self.mouse_btn = Some(bd.id);
                self.dev.button_down(bd.id, now);
                return;
            }
        }
        for (i, sx) in SLIDERS.iter().enumerate() {
            if (x - sx).abs() <= 12.0 && y >= SL_TOP - 14.0 && y <= SL_BOT + 14.0 {
                self.drag = Some(Drag::Slider(i));
                self.dev.set_slider(i, svg::slider_value(y));
                return;
            }
        }
        if (x - DIAL.0).hypot(y - DIAL.1) <= 92.0 {
            self.drag = Some(Drag::Dial((y - DIAL.1).atan2(x - DIAL.0).to_degrees()));
            return;
        }
        let (mx, my, mw, mh) = svg::MIC_RECT;
        if x >= mx && x <= mx + mw && y >= my && y <= my + mh {
            self.dev.pad_hit(now);
            return;
        }
        // anywhere else on the case: move the borderless window
        if !self.decorations {
            if let Some(sf) = &self.win {
                let _ = sf.window.drag_window();
            }
        }
    }

    /// Ctrl +/−/0: the borderless window has no frame to resize with.
    fn zoom(&mut self, z: Option<f64>) {
        self.zoom = z.unwrap_or(1.0).clamp(0.4, 2.0);
        if let Some(sf) = &self.win {
            let _ = sf.window.request_inner_size(LogicalSize::new(svg::W * self.zoom, svg::H * self.zoom));
        }
    }

    fn mouse_move(&mut self) {
        let (x, y) = self.to_panel(self.cursor);
        match self.drag {
            Some(Drag::Slider(i)) => self.dev.set_slider(i, svg::slider_value(y)),
            Some(Drag::Dial(prev)) => {
                let a = (y - DIAL.1).atan2(x - DIAL.0).to_degrees();
                if (x - DIAL.0).hypot(y - DIAL.1) < DIAL_DEAD_RADIUS {
                    // follow the angle without turning, so leaving the centre doesn't jump
                    self.drag = Some(Drag::Dial(a));
                    return;
                }
                let mut d = a - prev;
                if d > 180.0 {
                    d -= 360.0;
                }
                if d < -180.0 {
                    d += 360.0;
                }
                self.drag = Some(Drag::Dial(a));
                self.turn(d);
            }
            None => {}
        }
    }

    fn mouse_up(&mut self) {
        if let Some(b) = self.mouse_btn.take() {
            self.dev.button_up(b, Instant::now());
        }
        if matches!(self.drag, Some(Drag::Dial(_))) {
            // letting go: the knob settles on the detent it shows
            self.dial_acc = 0.0;
        }
        self.drag = None;
    }

    fn key(&mut self, code: KeyCode, pressed: bool, repeat: bool) {
        let now = Instant::now();
        match code {
            KeyCode::ArrowUp | KeyCode::ArrowRight if pressed => return self.turn(DETENT),
            KeyCode::ArrowDown | KeyCode::ArrowLeft if pressed => return self.turn(-DETENT),
            KeyCode::KeyK if pressed && !repeat => return self.dev.pad_hit(now),
            KeyCode::KeyC if pressed && !repeat => {
                // COUNT IN: [BEAT/PATTERN] + [VOICE] together
                self.dev.button_down(Btn::Voice, now);
                self.dev.button_down(Btn::Beat, now);
                self.dev.button_up(Btn::Beat, now);
                self.dev.button_up(Btn::Voice, now);
                return;
            }
            _ => {}
        }
        let b = match code {
            KeyCode::Space | KeyCode::Enter => Btn::Start,
            KeyCode::KeyT => Btn::Tap,
            KeyCode::KeyL => Btn::Light,
            KeyCode::KeyM => Btn::Mute,
            KeyCode::KeyB => Btn::Beat,
            KeyCode::KeyV => Btn::Voice,
            KeyCode::KeyP => Btn::Power,
            KeyCode::KeyW => Btn::Write,
            KeyCode::KeyE => Btn::Memory,
            KeyCode::Digit1 => Btn::Tempo,
            KeyCode::Digit2 => Btn::Loop,
            KeyCode::Digit3 => Btn::Tune,
            KeyCode::Digit4 => Btn::Pitch,
            KeyCode::Digit5 => Btn::Coach,
            KeyCode::Digit6 => Btn::Trig,
            KeyCode::Digit7 => Btn::Midi,
            _ => return,
        };
        if repeat {
            return;
        }
        if pressed {
            self.dev.button_down(b, now);
        } else {
            self.dev.button_up(b, now);
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.size = (w.max(1), h.max(1));
        let sf = self.win.as_ref().map(|w| w.window.scale_factor());
        debug(format_args!("resize {w}x{h} scale_factor {sf:?}"));
        let s = self.gfx_scale() as f32;
        let t = Instant::now();
        self.gfx.set_scale(s);
        debug(format_args!("set_scale {s:.3} took {:?}", t.elapsed()));
        let (pw, ph) = self.gfx.panel_px();
        self.offset = ((self.size.0 as i32 - pw as i32) / 2, (self.size.1 as i32 - ph as i32) / 2);
        if let Some(sf) = &mut self.win {
            let _ = sf.surface.resize(NonZeroU32::new(self.size.0).unwrap(), NonZeroU32::new(self.size.1).unwrap());
        }
        self.last_sig.clear();
    }

    /// Cheap summary of everything visible; a redraw happens only when it changes.
    fn signature(&self) -> String {
        let d = &self.dev;
        let mut h = DefaultHasher::new();
        svg::lcd(d).hash(&mut h);
        format!(
            "{}|{:?}|{:?}|{:?}|{:.1}|{:?}|{}|{}",
            h.finish(),
            d.leds,
            d.held,
            d.sliders,
            self.knob_angle,
            d.notice.map(|n| n.0),
            d.power,
            (d.light_level * 100.0) as i32 + (d.ink_level * 100.0) as i32 * 1000
        )
    }

    fn draw(&mut self) {
        self.sync_size();
        let Some(sf) = &mut self.win else { return };
        let mut held = self.dev.held.clone();
        if let Some(b) = self.mouse_btn {
            if !held.contains(&b) {
                held.push(b);
            }
        }
        let canvas = self.gfx.compose(&self.dev, self.knob_angle, &held);
        let Ok(mut buf) = sf.surface.buffer_mut() else { return };
        let (w, h) = self.size;
        let (pw, ph) = (canvas.width() as i32, canvas.height() as i32);
        let (ox, oy) = self.offset;
        let src = canvas.data();
        let transparent = self.transparent;
        let empty = if transparent { 0 } else { BG };
        for y in 0..h as i32 {
            let row = &mut buf[(y as usize * w as usize)..((y as usize + 1) * w as usize)];
            let py = y - oy;
            if py < 0 || py >= ph {
                row.fill(empty);
                continue;
            }
            for (x, px) in row.iter_mut().enumerate() {
                let cx = x as i32 - ox;
                *px = if cx < 0 || cx >= pw {
                    empty
                } else {
                    let i = ((py * pw + cx) * 4) as usize;
                    let a = src[i + 3] as u32;
                    if transparent {
                        // premultiplied ARGB straight to the compositor
                        a << 24 | (src[i] as u32) << 16 | (src[i + 1] as u32) << 8 | src[i + 2] as u32
                    } else {
                        // premultiplied RGBA over the window background
                        let mix = |c: u8, bg: u32| c as u32 + bg * (255 - a) / 255;
                        mix(src[i], BG >> 16) << 16 | mix(src[i + 1], (BG >> 8) & 255) << 8 | mix(src[i + 2], BG & 255)
                    }
                };
            }
        }
        let _ = buf.present();
        debug(format_args!("frame presented"));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        debug(format_args!("resumed"));
        if self.win.is_some() {
            return;
        }
        let mut size = LogicalSize::new(svg::W, svg::H);
        if let Some(m) = el.primary_monitor() {
            let ms = m.size().to_logical::<f64>(m.scale_factor());
            let k = (ms.height * 0.9 / svg::H).min(ms.width * 0.9 / svg::W).min(1.0);
            size = LogicalSize::new(svg::W * k, svg::H * k);
            self.zoom = k;
        }
        let attrs = Window::default_attributes()
            .with_title("Kadence KD-90")
            .with_window_icon(super::render::window_icon())
            .with_decorations(self.decorations)
            .with_transparent(self.transparent)
            .with_inner_size(size)
            .with_min_inner_size(LogicalSize::new(svg::W * 0.4, svg::H * 0.4));
        let attrs = with_app_id(attrs);
        let window = Rc::new(el.create_window(attrs).expect("create window"));
        let ctx = softbuffer::Context::new(window.clone()).expect("softbuffer context");
        let surface = softbuffer::Surface::new(&ctx, window.clone()).expect("softbuffer surface");
        let ps = window.inner_size();
        self.win = Some(Surface { window, surface });
        self.resize(ps.width, ps.height);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.dev.save();
                el.exit();
            }
            WindowEvent::Resized(s) => self.resize(s.width, s.height),
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(sf) = &self.win {
                    let s = sf.window.inner_size();
                    self.resize(s.width, s.height);
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = position;
                self.mouse_move();
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => match state {
                ElementState::Pressed => self.mouse_down(),
                ElementState::Released => self.mouse_up(),
            },
            WindowEvent::CursorLeft { .. } => self.mouse_up(),
            WindowEvent::MouseWheel { delta, .. } => {
                let notches = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(p) => {
                        self.wheel_acc += p.y / 40.0;
                        let n = self.wheel_acc.trunc();
                        self.wheel_acc -= n;
                        n
                    }
                };
                if notches != 0.0 {
                    self.turn(notches * DETENT);
                }
            }
            WindowEvent::ModifiersChanged(m) => self.ctrl = m.state().control_key() || m.state().super_key(),
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else { return };
                let pressed = event.state == ElementState::Pressed;
                match code {
                    // no title bar: Ctrl+Q closes the app (as does POWER)
                    KeyCode::KeyQ if pressed && self.ctrl => {
                        self.dev.save();
                        el.exit();
                    }
                    KeyCode::Equal | KeyCode::NumpadAdd if pressed && self.ctrl => self.zoom(Some(self.zoom + 0.1)),
                    KeyCode::Minus | KeyCode::NumpadSubtract if pressed && self.ctrl => self.zoom(Some(self.zoom - 0.1)),
                    KeyCode::Digit0 | KeyCode::Numpad0 if pressed && self.ctrl => self.zoom(None),
                    _ if self.ctrl => {}
                    _ => self.key(code, pressed, event.repeat),
                }
            }
            WindowEvent::Focused(false) => {
                // release everything held so keys don't get stuck
                for b in self.dev.held.clone() {
                    self.dev.button_up(b, Instant::now());
                }
                self.mouse_up();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        while let Ok(t) = self.hits.try_recv() {
            self.dev.pad_hit(t);
        }
        let now = Instant::now();
        self.dev.tick(now);
        if self.dev.powered_off() {
            el.exit();
            return;
        }
        if self.gfx.poll_background() {
            self.last_sig.clear();
        }
        let sig = self.signature();
        if sig != self.last_sig {
            self.last_sig = sig;
            if let Some(sf) = &self.win {
                sf.window.request_redraw();
            }
        }
        let dt = if self.dev.needs_animation() || self.drag.is_some() { 12 } else { 60 };
        el.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(dt)));
    }
}

/// Desktop app ID (also the .desktop file and icon name). Wayland has no window icons:
/// the compositor finds the icon through the installed `<APP_ID>.desktop` file.
const APP_ID: &str = "io.github.bewzaalex.kadence";

#[cfg(all(unix, not(target_os = "macos")))]
fn with_app_id(attrs: winit::window::WindowAttributes) -> winit::window::WindowAttributes {
    use winit::platform::{wayland::WindowAttributesExtWayland, x11::WindowAttributesExtX11};
    let attrs = WindowAttributesExtWayland::with_name(attrs, APP_ID, APP_ID);
    WindowAttributesExtX11::with_name(attrs, APP_ID, APP_ID)
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn with_app_id(attrs: winit::window::WindowAttributes) -> winit::window::WindowAttributes {
    attrs
}

/// `KADENCE_DEBUG=1`: timestamped diagnostics on stderr.
fn debug(msg: std::fmt::Arguments) {
    use std::sync::OnceLock;
    static START: OnceLock<Instant> = OnceLock::new();
    if std::env::var_os("KADENCE_DEBUG").is_some() {
        let t = START.get_or_init(Instant::now).elapsed();
        eprintln!("[{:7.3}s] {msg}", t.as_secs_f64());
    }
}
