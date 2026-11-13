# softbuffer 0.4.8 — Kadence patch

Upstream softbuffer drops the alpha channel. Kadence draws a borderless window whose
rounded case and shadow float over the desktop, so the alpha byte must reach the compositor.

Changes against the crates.io release:

- `src/backends/wayland/buffer.rs`: `wl_shm::Format::Xrgb8888` → `Argb8888` (2 places).
  Pixels are premultiplied `0xAARRGGBB`.
- `src/backends/cg.rs` (macOS): `CGImageAlphaInfo::NoneSkipFirst` → `PremultipliedFirst`.

X11 needs no change: softbuffer already accepts depth-32 (ARGB) visuals, which winit picks
for transparent windows.

Windows is not patched yet (GDI ignores alpha; needs a layered window) — see docs/TODO.md.
Apps using this copy must write a valid premultiplied alpha byte in every pixel.
