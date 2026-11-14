# TODO

## Mockup
- [ ] Logo / branding: a proper emblem with "k", the KADENCE wordmark, "METRONOME Dr.Tick" and
      KD-90 lettering. First attempt (rejected, kept for reference):
      `design/drafts/logo-draft-panel.js` → `branding()`.

## App
- [x] Linux audio: PulseAudio protocol (works with PipeWire's pipewire-pulse) with ALSA fallback.
- [ ] Linux output latency over PulseAudio/pipewire-pulse is ~190–220 ms here (server minimum; we ask
      for 10 ms). Clicks stay sample-accurate and the pendulum/LEDs/Time Check are compensated, but
      START/STOP and MUTE are heard ~0.2 s after the press. Option: optional `pipewire` cargo feature
      (native, ~10–20 ms; needs libpipewire + clang to build).
- [ ] Windows: transparent borderless window. softbuffer's GDI path ignores alpha, so Windows
      currently shows the borderless case on a dark square. Implement a layered window
      (WS_EX_LAYERED + UpdateLayeredWindow with a premultiplied 32-bit DIB) in vendor/softbuffer
      or as a Windows-only present path in the app.
- [ ] Windows release package: add a Windows job to `.github/workflows/release.yml` (zip or
      installer), after the transparent window and the .exe icon below are done and tested.
- [ ] App icon outside Linux: macOS needs an .app bundle with an .icns for the Dock; Windows needs
      the icon embedded as an .exe resource (build script) for Explorer and the taskbar.
- [ ] Verify transparency on X11 (softbuffer accepts depth-32 visuals; untested) and macOS.
- [ ] Voice 4 is a synthesized stand-in; replace with a recorded human count.
- [ ] Drum sounds and patterns are synthesized; tune them by ear.
- [ ] Memory: ~45–100 MB RSS at HiDPI (resvg filter/mask layers when rasterizing the background,
      two full-size frame buffers). Ideas: render background without big blur/mask layers,
      draw directly into the softbuffer, share one frame buffer.
- [ ] Resizing: every new window size redraws the whole background (a blurry preview first, the
      sharp panel about a second later), and each size is cached separately. Rework it, e.g. draw
      the sharp background once at the largest useful size and scale it down while resizing, or
      redraw only after the size has settled.
- [ ] Flatpak: move to the freedesktop 26.08 runtime (the linter warns that 25.08 is outdated);
      needs the 26.08 Rust SDK extension.
- [ ] Volume sliders: decide whether they should also move in clicked steps (like the detented
      TEMPO/VALUE dial) instead of the current smooth analog travel.
- [ ] Time Check accuracy depends on audio-output/mic latency estimates; add a latency calibration.

## Later
- [ ] MIDI IN sync (MIDI mode): receive Timing Clock F8, Start FA, Continue FB, Stop FC,
      Active Sensing FE; tempo follows the clock; tempo dial/TAP disabled; MIDI cable loss stops
      playback. Until then the MIDI button is inert and shows a "not implemented yet" notice.
