// Kadence lettering for the screen panel: three styles in one justified block.
//
//   KADENCE            heavy blocks whose counters are thin slits
//   METRONOME Dr.Tick  small, wide, rounded monoline
//   KD-90              large, wide, rounded monoline, medium weight
//
// Every glyph is drawn on a cap height of 10 units; `block()` scales and spaces each line so all
// three share one left and one right edge. Works in the browser (window.KadenceLettering) and in
// node (module.exports), so the mockup previews and the app use the same shapes.
(function (root) {
  'use strict';
  const f = (v) => Math.round(v * 100) / 100;

  // ---------------------------------------------------------------- heavy slit letters (filled)
  // Paths are filled with the even-odd rule: the slits are sub-paths cut out of each block.
  const S = 1.15; // slit width
  const slitY = (y) => `${f(y - S / 2)}`;
  const HEAVY = {
    K: { w: 10.4, d: 'M0,0H4V3.7L6.8,0H10.6L6.9,5L10.6,10H6.8L4,6.3V10H0Z' },
    A: { w: 10.6, d: `M0,10V3A3,3 0 0 1 3,0H7.6A3,3 0 0 1 10.6,3V10H5.88V6.4H4.72V10Z M3.9,${slitY(3.9)}H6.7V${f(3.9 + S / 2)}H3.9Z` },
    D: { w: 10.2, d: `M0,0H7.2A3,3 0 0 1 10.2,3V7A3,3 0 0 1 7.2,10H0Z M3.9,${slitY(5)}H7V${f(5 + S / 2)}H3.9Z` },
    E: { w: 9.4, d: `M0,0H9.4V10H0Z M3.9,${slitY(3.45)}H9.4V${f(3.45 + S)}H3.9Z M3.9,${slitY(6.55)}H9.4V${f(6.55 + S)}H3.9Z` },
    N: { w: 10.6, d: 'M0,0H5.1L6.7,4.4V0H10.6V10H5.5L3.9,5.6V10H0Z' },
    C: { w: 10.2, d: `M10.2,0H3A3,3 0 0 0 0,3V7A3,3 0 0 0 3,10H10.2V${f(5 + S / 2)}H3.9V${slitY(5)}H10.2Z` },
  };

  // ---------------------------------------------------------------- rounded monoline (stroked)
  // Centre lines inside a w × 10 box; `s` is the stroke width, `r` the corner radius.
  function rrect(x0, y0, x1, y1, r) {
    return `M${f(x0 + r)},${f(y0)}H${f(x1 - r)}A${r},${r} 0 0 1 ${f(x1)},${f(y0 + r)}V${f(y1 - r)}` +
      `A${r},${r} 0 0 1 ${f(x1 - r)},${f(y1)}H${f(x0 + r)}A${r},${r} 0 0 1 ${f(x0)},${f(y1 - r)}` +
      `V${f(y0 + r)}A${r},${r} 0 0 1 ${f(x0 + r)},${f(y0)}Z`;
  }
  function mono(ch, s, wide = 1) {
    const h = s / 2, t = h, b = 10 - h, m = 5, xh = 3.4; // top, bottom, middle, x-height top
    const r = 2.6;
    const W = {
      M: 11.5, E: 8.6, T: 8.6, R: 9.6, O: 11.5, N: 10.2, K: 10.6, D: 11.2, '-': 4.6, 9: 11.2, 0: 11.2,
      r: 6.4, '.': 1.8, i: 1.8, c: 7.6, k: 7.6, ' ': 3.6,
    };
    const w = W[ch] * (ch === ' ' || ch === '.' || ch === 'i' ? 1 : wide);
    const L = h, R = w - h; // left, right centre lines
    let d = '', round = false;
    switch (ch) {
      case 'M': d = `M${L},${b}V${t + r}A${r},${r} 0 0 1 ${L + r},${t}H${w / 2 - r}A${r},${r} 0 0 1 ${w / 2},${t + r}V${b}` +
        `M${w / 2},${t + r}A${r},${r} 0 0 1 ${w / 2 + r},${t}H${R - r}A${r},${r} 0 0 1 ${R},${t + r}V${b}`; break;
      case 'E': d = `M${R},${t}H${L + r}A${r},${r} 0 0 0 ${L},${t + r}V${b - r}A${r},${r} 0 0 0 ${L + r},${b}H${R}M${L},${m}H${R - 1}`; break;
      case 'T': d = `M${L - h},${t}H${R + h}M${w / 2},${t}V${b}`; break;
      case 'R': d = `M${L},${b}V${t}H${R - r}A${r},${r} 0 0 1 ${R},${t + r}V${m - r}A${r},${r} 0 0 1 ${R - r},${m}H${L}` +
        `M${R - r},${m}A${r},${r} 0 0 1 ${R},${m + r}V${b}`; break;
      case 'O': case '0': d = rrect(L, t, R, b, ch === 'O' ? r : r * 1.15); break;
      case 'N': d = `M${L},${b}V${t + r}A${r},${r} 0 0 1 ${L + r},${t}H${R - r}A${r},${r} 0 0 1 ${R},${t + r}V${b}`; break;
      // a playful K: the arm rises from just below the middle of the stem to the top corner, the
      // leg branches off the arm (not the stem) down to the bottom corner; ends are round
      case 'K': {
        const ay = m + 0.9, px = L + (R - L) * 0.38, py = ay + (t - ay) * 0.38;
        d = `M${L},${t}V${b}M${L},${f(ay)}L${R},${t}M${f(px)},${f(py)}L${R},${b}`;
        round = true;
        break;
      }
      // closed: a gap between stem and bowl (as some styles do) would read as "I" after the K
      case 'D': {
        const q = r * 1.3;
        d = `M${L},${t}H${R - q}A${q},${q} 0 0 1 ${R},${t + q}V${b - q}A${q},${q} 0 0 1 ${R - q},${b}H${L}Z`;
        break;
      }
      case '-': d = `M${L},${m}H${R}`; break;
      case '9': d = rrect(L, t, R, m + 0.6, r * 1.15) + `M${R},${m}V${b - r * 1.15}A${r * 1.15},${r * 1.15} 0 0 1 ${R - r * 1.15},${b}H${L}`; break;
      case 'r': d = `M${L},${b}V${xh + r * 0.8}A${r * 0.8},${r * 0.8} 0 0 1 ${L + r * 0.8},${xh}H${R}`; break;
      case '.': d = `M${w / 2},${b - 0.05}V${b}`; break;
      case 'i': d = `M${w / 2},${xh}V${b}M${w / 2},${t - 0.05}V${t}`; break;
      case 'c': d = `M${R},${xh}H${L + r * 0.8}A${r * 0.8},${r * 0.8} 0 0 0 ${L},${xh + r * 0.8}V${b - r * 0.8}` +
        `A${r * 0.8},${r * 0.8} 0 0 0 ${L + r * 0.8},${b}H${R}`; break;
      case 'k': {
        const ay = (xh + b) / 2 + 0.6, px = L + (R - L) * 0.38, py = ay + (xh - ay) * 0.38;
        d = `M${L},${t}V${b}M${L},${f(ay)}L${R},${xh}M${f(px)},${f(py)}L${R},${b}`;
        round = true;
        break;
      }
      case ' ': d = ''; break;
      default: throw new Error('no glyph ' + ch);
    }
    return { w, d, round };
  }

  // ---------------------------------------------------------------- lines and the block
  // A line of glyphs at cap height `cap`, spread to exactly `width` by tracking.
  function line(text, cap, width, kind, opt = {}) {
    const glyphs = [...text].map((ch) => (kind === 'heavy' ? HEAVY[ch] : mono(ch, opt.stroke, opt.wide)));
    const natural = glyphs.reduce((a, g) => a + g.w, 0);
    const k = cap / 10;
    const gaps = glyphs.length - 1;
    const track = (width / k - natural) / gaps; // in glyph units
    let x = 0, o = '';
    for (const g of glyphs) {
      if (g.d) o += `<path transform="translate(${f(x)},0)"${g.round ? ' stroke-linecap="round"' : ''} d="${g.d}"/>`;
      x += g.w + track;
    }
    const style = kind === 'heavy'
      ? 'fill-rule="evenodd"'
      : `fill="none" stroke-width="${opt.stroke}" stroke-linejoin="round" stroke-linecap="square"`;
    return { svg: `<g transform="scale(${f(k)})" ${style}>${o}</g>`, track: f(track) };
  }

  // The whole block: left edge x, top y, width w, in panel units; returns SVG and its height.
  function block(x, y, w, ink) {
    const capA = w / 7.85, capB = w / 15.5, capC = w / 6.2; // KADENCE, METRONOME line, KD-90
    const gapAB = capA * 0.75, gapBC = capB * 0.95;
    const yB = y + capA + gapAB, yC = yB + capB + gapBC;
    const a = line('KADENCE', capA, w, 'heavy');
    const b = line('METRONOME Dr.Tick', capB, w, 'mono', { stroke: 2.0 });
    const c = line('KD-90', capC, w, 'mono', { stroke: 1.95, wide: 1.2 });
    const svg = `<g fill="${ink}" stroke="${ink}">` +
      `<g transform="translate(${f(x)},${f(y)})" stroke="none">${a.svg}</g>` +
      `<g transform="translate(${f(x)},${f(yB)})">${b.svg}</g>` +
      `<g transform="translate(${f(x)},${f(yC)})">${c.svg}</g></g>`;
    return { svg, height: yC + capC - y, tracking: { kadence: a.track, metronome: b.track, model: c.track } };
  }

  const api = { block, HEAVY, mono };
  if (typeof module !== 'undefined') module.exports = api;
  else root.KadenceLettering = api;
})(this);
