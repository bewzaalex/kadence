//! Kadence KD-90: a rhythm-trainer metronome.

mod audio;
mod device;
mod dirs;
mod ui;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use winit::event_loop::EventLoop;

use device::Device;
use ui::panel::BUTTONS;

/// `kadence --render out.png [button… | dial:N…]`: renders the panel to a PNG without opening a
/// window (pressing the named buttons and turning the dial first), for previews and visual checks.
fn render_snapshot(dev: &mut Device, args: &[String]) {
    let now = Instant::now();
    for name in &args[1..] {
        // dial:N turns the dial N clicks (negative = counter-clockwise)
        if let Some(n) = name.strip_prefix("dial:").and_then(|n| n.parse().ok()) {
            dev.dial(n, now);
        } else if let Some(bd) = BUTTONS.iter().find(|b| format!("{:?}", b.id).eq_ignore_ascii_case(name)) {
            dev.button_down(bd.id, now);
            dev.button_up(bd.id, now);
        }
    }
    // KADENCE_AT_MS renders the panel that many milliseconds after the presses (animations)
    let at = std::env::var("KADENCE_AT_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    dev.tick(now + Duration::from_millis(at));
    let mut g = ui::render::Gfx::new();
    g.set_scale(std::env::var("KADENCE_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(2.0));
    g.finish_background();
    let t = Instant::now();
    let pm = g.snapshot(dev, 0.0);
    if let Err(e) = pm.save_png(&args[0]) {
        eprintln!("kadence: {e}");
    }
    eprintln!("rendered {} in {:?}", args[0], t.elapsed());
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let audio = audio::Audio::new(tx);
    let mut dev = Device::new(audio);
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() >= 2 && args[0] == "--render" {
        return render_snapshot(&mut dev, &args[1..]);
    }
    let decorations = args.iter().any(|a| a == "--decorations");
    // see-through corners where the graphics system allows it (else the window background shows)
    let transparent = !decorations;
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = ui::app::App::new(dev, rx, decorations, transparent);
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("kadence: {e}");
    }
}
