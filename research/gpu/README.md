# Research: drawing Kadence on the GPU

Kadence draws its whole panel on the CPU and hands the pixels to the window with a patched
softbuffer. This spike compares two ways to let the GPU put the picture on screen instead:

- **wgpu** (winit + wgpu 30): Vulkan on Linux, Metal on macOS, DX12 on Windows
- **GL** (winit + glutin 0.32 + glow 0.18): OpenGL 3.3 core, or OpenGL ES 3.0

Both programs do the same work as Kadence: the panel is a picture made once (here a 2x PNG from
`kadence --render`, with mipmaps), and the moving parts are drawn by the GPU on every frame over
it: a swinging needle at 120 BPM, 27 fan bars on the LCD and a marker turning on the dial. The
window is borderless and see-through, like Kadence's.

## Running

```sh
KADENCE_SCALE=2 ../../target/release/kadence --render panel.png   # the panel picture
cargo run --release --features wgpu --bin spike-wgpu
cargo run --release --features gl --bin spike-gl
```

Esc or Q quits, dragging moves the window. Environment variables:

| Variable | Effect |
| --- | --- |
| `SPIKE_SECONDS=N` | quit after N seconds and print a summary |
| `SPIKE_RESIZE=1` | the window grows and shrinks all the time |
| `SPIKE_NO_NOTIFY=1` | don't wait for the compositor's frame callback before presenting |
| `SPIKE_PRESENT=mailbox` | wgpu only: present mode `mailbox` or `immediate` (default `fifo`) |
| `WGPU_DX12_PRESENTATION_SYSTEM=visual` | wgpu on Windows: the swapchain that keeps alpha |

## Results (2026-11-15, KDE Plasma Wayland, AMD Radeon 780M, Mesa 26.1)

| | wgpu | GL | Kadence now |
| --- | --- | --- | --- |
| Transparent window, Wayland | yes (PreMultiplied) | yes (8-bit alpha config) | yes, patched softbuffer |
| Transparent window, X11 | yes | yes | yes |
| Transparent window, Windows | DX12 with a DirectComposition swapchain, supported by wgpu | WGL + winit's DWM blur-behind trick | no |
| Transparent window, macOS | Metal, supported | OpenGL, deprecated by Apple since 2018 | patched softbuffer |
| First frame after start | 85–100 ms | 75–85 ms | |
| CPU while animating | 3.5 % | 2.6 % | |
| Memory (RSS) | 61 MB | 70 MB | 50 MB |
| Release binary, stripped | 7.5 MB | 3.1 MB | 7.1 MB |
| Crates | 156 | 103 | 173 |
| Clean release build | 42 s | 23 s | |
| Lines of code (backend only) | 235 | 187 | |
| Unsafe code | none (one byte cast) | every GL call | |

Each figure is the mean of two to three runs. RSS includes the GPU driver: Mesa's Vulkan driver
(RADV) is smaller than its OpenGL driver (radeonsi), which is why wgpu uses less memory here.
Another GPU or system will give different numbers.

Both ran at 45 fps on a 60 Hz screen. This is the compositor: KWin sends the window's frame
callbacks at 45 per second, and Vulkan's FIFO and EGL's swap interval both wait for them. Kadence
already works around it with its own steady frame clock. With mailbox present and no frame
callback, wgpu presents as fast as asked.

Notes:

- On this system wgpu first offered `Rgb10a2Unorm`, which has 2 bits of alpha. The shadows would
  show bands, so the spike asks for an 8-bit format (`Bgra8Unorm`) by name.
- wgpu changes its API with each major version, about every three months. Going from 25 to 30 here
  needed four small changes: the display handle, the surface colour space, `Queue::present` and
  optional vertex buffers.

## Decision: wgpu

1. **Every platform has a supported way to the transparent window.**
   - wgpu offers it on Vulkan/Wayland, X11, Metal and DX12 (through DirectComposition).
   - GL has none on macOS, where Apple has deprecated OpenGL. On Windows, GL relies on a DWM trick.
   - The future-proof route is the one each platform still develops: Vulkan, Metal and DX12.
2. **Safe code with checks.**
   - Every wgpu call is checked: a mistake becomes a clear error message.
   - In GL, every call is `unsafe`, and a mistake shows up as a black or garbled window.
3. **Widely used.**
   - wgpu is behind Firefox's WebGPU, Bevy, iced and many others, so its bugs are found and fixed
     quickly.
   - glutin and glow are stable but have few maintainers.
4. **It costs:**
   - About 4 MB more binary than GL.
   - Twice the build time.
   - Upgrading every few releases. Kadence can stay on one version for a long time and upgrade when
     it chooses.
   - Speed and memory were close; wgpu even used less memory here.

GL would be the choice for the smallest binary on Linux only. Kadence needs Windows and macOS too.

## Done

Kadence now draws with wgpu (src/ui/gpu.rs): the vendored softbuffer is gone, the window shows
the old pictures scaled while resizing, and layer pixels are freed once they are textures
(RSS about 59 MB, was 50 MB).

Found on the way: wgpu's OpenGL fallback offers only opaque surfaces (`composite_alpha_modes:
[Opaque] // TODO` in wgpu-hal), so without Vulkan the case sits on a dark square.

## Still to check

- Windows: the transparent window with `DxgiFromVisual`, on real hardware.
- macOS: the transparent window on Metal.
- An integrated GPU without Vulkan, or a virtual machine: wgpu falls back to its own GL backend
  there. Check the transparency in that case.
