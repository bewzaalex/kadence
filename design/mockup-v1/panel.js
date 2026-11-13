// Kadence KD-90 mockup — SVG panel + LCD renderer.
// Pure string builders so the same code runs in the browser and in node (for previews).
(function (root) {
  'use strict';

  // ---------- 7-segment glyphs ----------
  // bit order: a b c d e f g
  const GLYPH = {
    '0': 'abcdef', '1': 'bc', '2': 'abdeg', '3': 'abcdg', '4': 'bcfg', '5': 'acdfg',
    '6': 'acdefg', '7': 'abc', '8': 'abcdefg', '9': 'abcdfg', '-': 'g', ' ': '',
    'A': 'abcefg', 'b': 'cdefg', 'C': 'adef', 'c': 'deg', 'd': 'bcdeg', 'E': 'adefg',
    'F': 'aefg', 'G': 'acdef', 'h': 'cefg', 'H': 'bcefg', 'L': 'def', 'n': 'ceg',
    'o': 'cdeg', 'P': 'abefg', 'q': 'abcfg', 'r': 'eg', 'S': 'acdfg', 't': 'defg',
    'u': 'cde', 'U': 'bcdef', 'y': 'bcdfg',
  };

  const f1 = (n) => Math.round(n * 10) / 10;
  const pts = (arr) => arr.map((p) => f1(p[0]) + ',' + f1(p[1])).join(' ');

  function hseg(x1, x2, y, t) {
    const h = t / 2;
    return [[x1, y], [x1 + h, y - h], [x2 - h, y - h], [x2, y], [x2 - h, y + h], [x1 + h, y + h]];
  }
  function vseg(x, y1, y2, t) {
    const h = t / 2;
    return [[x, y1], [x + h, y1 + h], [x + h, y2 - h], [x, y2], [x - h, y2 - h], [x - h, y1 + h]];
  }
  function segShapes(w, h, t) {
    const g = t * 0.18;
    const xl = t / 2, xr = w - t / 2, yt = t / 2, ym = h / 2, yb = h - t / 2;
    return {
      a: hseg(xl + g, xr - g, yt, t), g: hseg(xl + g, xr - g, ym, t), d: hseg(xl + g, xr - g, yb, t),
      f: vseg(xl, yt + g, ym - g, t), b: vseg(xr, yt + g, ym - g, t),
      e: vseg(xl, ym + g, yb - g, t), c: vseg(xr, ym + g, yb - g, t),
    };
  }

  // Draws a digit; `mask` is a string of lit segment letters. Unlit segments are ghosts.
  function digitMask(x, y, w, h, t, mask, visible = true) {
    const s = segShapes(w, h, t);
    let o = `<g transform="translate(${f1(x)},${f1(y)}) skewX(-7)">`;
    for (const k of 'abcdefg') {
      const lit = visible && mask.includes(k);
      o += `<polygon class="${lit ? 'on' : 'gh'}" points="${pts(s[k])}"/>`;
    }
    return o + '</g>';
  }
  const digit = (x, y, w, h, t, ch, visible = true) =>
    digitMask(x, y, w, h, t, GLYPH[ch] ?? '', visible);

  // Row of digits, right-aligned text into `cells` positions.
  function digits(xs, y, w, h, t, text, visible = true) {
    const s = String(text).padStart(xs.length, ' ').slice(-xs.length);
    return xs.map((x, i) => digit(x, y, w, h, t, s[i], visible)).join('');
  }

  // ---------- LCD ----------
  const BIG = { w: 22, h: 38, t: 4.4, xs: [92, 118, 144], y: 7 };
  const SM = { w: 11, h: 19, t: 2.5 };
  const MEM_XS = [22, 36];
  const RIGHT_XS = [194, 208, 222];
  const PIVOT = [125, 150];
  const PEND_N = 17, PEND_SPAN = 44;

  const label = (txt, x, y, on = true, size = 6.6, anchor = 'start') =>
    `<text class="${on ? 'lt' : 'lg'}" x="${x}" y="${y}" font-size="${size}" text-anchor="${anchor}">${txt}</text>`;

  function pendulum(idx, on) {
    let o = '';
    for (let i = 0; i < PEND_N; i++) {
      const a = ((-PEND_SPAN + (2 * PEND_SPAN * i) / (PEND_N - 1)) * Math.PI) / 180;
      const r0 = 20, r1 = 90, w0 = 3.2, w1 = 1.3;
      const sx = Math.sin(a), cy = -Math.cos(a), px = Math.cos(a), py = Math.sin(a);
      const P = (r, w) => [PIVOT[0] + sx * r + px * w, PIVOT[1] + cy * r + py * w];
      const poly = [P(r0, -w0), P(r1, -w1), P(r1 + 2, 0), P(r1, w1), P(r0, w0)];
      o += `<polygon class="${on && i === idx ? 'on' : 'gh'}" points="${pts(poly)}"/>`;
    }
    return o;
  }

  // 5-step level wedge (like a signal-strength meter)
  function wedge(x, bottom, level, on) {
    let o = '';
    for (let i = 0; i < 5; i++) {
      const hh = 3 + i * 2.8;
      o += `<rect class="${on && i < level ? 'on' : 'gh'}" x="${f1(x + i * 4.6)}" y="${f1(bottom - hh)}" width="3.6" height="${f1(hh)}"/>`;
    }
    return o;
  }

  // Note glyphs above the rhythm sliders
  function noteIcon(kind, x, y) {
    const head = (cx, cy) => `<ellipse class="on" cx="${cx}" cy="${cy}" rx="2.6" ry="1.9" transform="rotate(-22 ${cx} ${cy})"/>`;
    const stem = (cx, cy) => `<rect class="on" x="${cx + 1.9}" y="${cy - 10}" width="0.9" height="10"/>`;
    const flag = (cx, cy, n) => {
      let o = '';
      for (let i = 0; i < n; i++) o += `<path class="on" d="M${cx + 2.8},${cy - 10 + i * 2.6} q3,2 3,5 q-0.8,-2.4 -3,-3.2z"/>`;
      return o;
    };
    const rest8 = (rx, ry) => `<path class="on" d="M${rx},${ry} a1.2,1.2 0 1,1 1.6,1.1 l2.4,-1.1 l-2.6,7 h-1 l2.2,-5.6 l-2.6,0.6z"/>`;
    const note = (cx, cy, flags) => head(cx, cy) + stem(cx, cy) + flag(cx, cy, flags);
    switch (kind) {
      case 1: return note(x + 4, y);
      case 2: return rest8(x, y - 9) + note(x + 9, y, 1);
      case 3: return rest8(x - 3, y - 9) + note(x + 5, y, 2) + rest8(x + 11, y - 9) + note(x + 19, y, 2);
      case 4: {
        let o = '';
        for (let i = 0; i < 3; i++) o += head(x + i * 6, y) + stem(x + i * 6, y);
        o += `<rect class="on" x="${x + 1.9}" y="${y - 10}" width="12.9" height="1.6"/>`;
        return o + label('3', x + 8, y - 12, true, 5, 'middle');
      }
    }
    return '';
  }

  // Sun icon: Auto Backlight Off
  function sun(cx, cy, on) {
    let o = `<circle class="${on ? 'on' : 'gh'}" cx="${cx}" cy="${cy}" r="2.4"/>`;
    for (let i = 0; i < 8; i++) {
      const a = (i * Math.PI) / 4;
      o += `<rect class="${on ? 'on' : 'gh'}" x="${f1(cx - 0.5)}" y="${f1(cy - 6)}" width="1" height="2.2" transform="rotate(${i * 45} ${cx} ${cy})"/>`;
      void a;
    }
    return o;
  }

  const NOTE_NAMES = ['C', 'C#', 'd', 'd#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'b'];

  // s: display state (see index.html for fields)
  function lcd(s) {
    const blinkHide = (field) => s.edit === field && !s.blink;
    const on = s.power;
    let o = `<rect x="0" y="0" width="250" height="200" fill="url(#${s.backlight && on ? 'lcdOn' : 'lcdOff'})"/>`;
    o += `<g class="${s.backlight && on ? 'bl' : 'nb'}">`;

    if (!on) {
      // power off: ghost layout only
      o += digits(BIG.xs, BIG.y, BIG.w, BIG.h, BIG.t, '   ', false) + pendulum(-1, false);
      return o + '</g>';
    }

    const mode = s.mode;
    const showPend = mode !== 'tune';

    // --- top-left: memory / sens ---
    if (mode === 'coach') {
      o += label('TRIG', 6, 12) + label('SENS', 6, 30);
      o += digits(MEM_XS, 20, SM.w, SM.h, SM.t, s.sens, !blinkHide('sens'));
      o += `<rect class="on" x="50" y="31" width="27" height="9" rx="1"/><text class="inv" x="63.5" y="38" font-size="6.6" text-anchor="middle">COACH</text>`;
      o += label('TRIG', 6, 52, true, 5.6) + `<circle class="${s.trig ? 'on' : 'gh'}" cx="14" cy="59" r="2.6"/>`;
    } else {
      o += label('MEMORY', 6, 15, !blinkHide('memory') || mode === 'tune');
      o += digits(MEM_XS, 20, SM.w, SM.h, SM.t, mode === 'tune' ? s.toneMemory : s.memory,
        !(blinkHide('memory') || blinkHide('write')));
      if (s.edited && mode !== 'tune') o += label('EDIT', 40, 47, true, 5.6);
    }

    // --- mode label next to memory ---
    if (mode === 'metro') o += label('TEMPO', 52, 18, !blinkHide('tempo'));
    if (mode === 'midi') o += label('MIDI', 56, 18) + `<path class="on" d="M60,2 l3,4 l3,-4z"/>`;
    if (mode === 'loop') o += label('LOOP', 56, 18) + `<path class="on" d="M60,2 l3,4 l3,-4z"/>`;
    if (mode === 'tune') o += label('NOTE', 56, 32, !blinkHide('note'));

    // --- big digits ---
    if (mode === 'tune') {
      const nn = NOTE_NAMES[s.note % 12];
      const oct = Math.floor(s.note / 12) - 1;
      const vis = !blinkHide('note');
      o += digit(BIG.xs[0], BIG.y, BIG.w, BIG.h, BIG.t, nn[0], vis);
      o += digit(BIG.xs[1], BIG.y, BIG.w, BIG.h, BIG.t, String(oct), vis);
      o += digit(BIG.xs[2], BIG.y, BIG.w, BIG.h, BIG.t, ' ', false);
      o += `<text class="${vis && nn[1] ? 'lt' : 'lg'}" x="114" y="13" font-size="10">♯</text>`;
    } else if (mode === 'coach' && s.timeCheck != null) {
      // 7-position timing indicator across the 3 big digits (3 = on the beat)
      const masks = ['', '', ''];
      const P = [[0, 'f'], [0, 'b'], [1, 'f'], [1, 'abf'], [1, 'b'], [2, 'f'], [2, 'b']];
      const [di, sg] = P[s.timeCheck];
      masks[di] += sg;
      masks[1] += 'd';
      o += BIG.xs.map((x, i) => digitMask(x, BIG.y, BIG.w, BIG.h, BIG.t, masks[i])).join('');
      o += label('[GOOD]', 129, 53, s.timeCheck === 3, 6.4, 'middle');
    } else {
      let txt;
      if (mode === 'loop') txt = s.loopRepeats;
      else if (mode === 'coach' && s.showCoachName) txt = ['tch', 'qct', 'Gud', 'Sud'][s.coach];
      else txt = s.tempo;
      o += digits(BIG.xs, BIG.y, BIG.w, BIG.h, BIG.t, txt, !blinkHide('tempo') && !blinkHide('repeats'));
    }

    // --- right block: BEAT/PATTERN + STYLE / PITCH ---
    if (mode === 'tune') {
      o += label('PITCH', 191, 40, !blinkHide('pitch'), 6.2, 'end');
      o += digits(RIGHT_XS, 22, SM.w, SM.h, SM.t, s.pitch, !blinkHide('pitch'));
      o += label('Hz', 236, 18, true, 5.6);
      if (!s.toneOn) o += label('MUTE', 246, 78, true, 6.6, 'end');
    } else {
      const isPat = s.beatMode === 'pattern';
      const bp = isPat ? label('PATTERN', 169, 36, !blinkHide('beat'), 5.4) : label('BEAT', 174, 32, !blinkHide('beat'));
      o += bp + label('STYLE', 202, 16);
      let st;
      let vis = true;
      if (mode === 'loop') st = s.loopCount;
      else if (mode === 'coach' && s.edit === 'guide') { st = `${s.guide} ${s.beat1}`; vis = s.blink; o += label('GUIDE', 174, 46); }
      else if (s.edit === 'countin' || (s.countIn > 0 && !s.playing)) { st = '-' + s.countIn; vis = !blinkHide('countin'); }
      else if (isPat) { st = s.pattern; vis = !blinkHide('beat'); }
      else if (s.playing) st = s.beatPos;
      else if (s.beat2 > 0) st = `${s.beat1}${s.beat2}`;
      else { st = s.beat1; vis = !blinkHide('beat'); }
      if (s.edit === 'beat2') { st = `${s.beat1}${s.beat2 || '-'}`; vis = s.blink; }
      o += digits(RIGHT_XS, 22, SM.w, SM.h, SM.t, String(st).replace(' ', '  '), vis);
      o += `<path class="on" d="M215,50 l3,-4 l3,4z"/>`;
      o += label(s.countIn > 0 && mode !== 'loop' ? 'COUNT IN' : 'COUNT', 218, 58, true, 6.2, 'middle');
      if (s.muted) o += label('MUTE', 246, 78, true, 6.6, 'end');
    }

    // --- voice ---
    if ((mode === 'metro' || mode === 'midi') && s.beatMode === 'beat') {
      o += label('VOICE', 6, 66, !blinkHide('voice'));
      o += digit(13, 70, SM.w, SM.h, SM.t, String(s.voice), !blinkHide('voice'));
    }

    o += sun(14, 106, !s.autoLightOff); // shown while Auto Backlight Off is disabled

    // --- pendulum ---
    if (showPend) o += pendulum(s.pend, true);

    // --- bottom: rhythm levels + VOL ---
    if (mode !== 'tune') {
      o += `<rect class="on" x="4" y="154" width="214" height="0.9"/>`;
      o += label('VOL', 230, 156, true, 6, 'middle');
      const isPat = s.beatMode === 'pattern';
      const xs = [8, 46, 84, 122, 160];
      const bottom = isPat ? 188 : 196;
      if (isPat) {
        ['KICK', 'SNARE', 'HH', 'OTHER1', 'OTHER2'].forEach((t, i) => (o += label(t, xs[i] + 10, 198, true, 6, 'middle')));
      } else {
        o += label('ACCENT', 6, 166, true, 6);
        o += noteIcon(1, 54, 174) + noteIcon(2, 88, 174) + noteIcon(3, 124, 174) + noteIcon(4, 164, 174);
      }
      xs.forEach((x, i) => (o += wedge(x + (isPat ? 0 : 4), bottom, s.levels[i], true)));
      const v = Math.ceil(s.vol / 20);
      for (let i = 0; i < 5; i++) {
        o += `<rect class="${i < v ? 'on' : 'gh'}" x="221" y="${f1(192 - i * 7.2)}" width="18" height="5.6"/>`;
      }
    }
    return o + '</g>';
  }

  // ---------- Panel ----------
  function button(id, cx, cy, w, h, lbl, style = 'plain', size = 9.6, rim = false) {
    const x = cx - w / 2, y = cy - h / 2;
    const lines = lbl.split('/');
    const multi = lines.length > 1 && style !== 'single';
    let t = '';
    const fill = style === 'light' ? '#1a1a1a' : '#f2f2f2';
    if (multi) {
      t = `<text x="${cx}" y="${cy - 1.5}" font-size="${size - 0.8}" text-anchor="middle" fill="${fill}">${lines[0]}/</text>` +
          `<text x="${cx}" y="${cy + 8}" font-size="${size - 0.8}" text-anchor="middle" fill="${fill}">${lines[1]}</text>`;
    } else {
      t = `<text x="${cx}" y="${cy + size * 0.36}" font-size="${size}" text-anchor="middle" fill="${fill}">${lbl}</text>`;
    }
    let deco = '';
    const bw = Math.min(w - 10, lbl.length * size * 0.66 + 8);
    const bh = multi ? 22 : size + 6;
    if (style === 'box') deco = `<rect x="${cx - bw / 2}" y="${cy - bh / 2}" width="${bw}" height="${bh}" rx="2" fill="none" stroke="#e8e8e8" stroke-width="1.1"/>`;
    if (style === 'light') deco = `<rect x="${cx - bw / 2}" y="${cy - bh / 2}" width="${bw}" height="${bh}" rx="2" fill="#d6d8da"/>`;
    let o = '';
    // raised bezel (POWER / WRITE) so they can be found blind
    if (rim) {
      o += `<rect x="${x - 6.5}" y="${y - 6.5}" width="${w + 13}" height="${h + 13}" rx="8.5" fill="url(#chamferBig)"/>` +
        `<rect x="${x - 6.5}" y="${y - 6.5}" width="${w + 13}" height="${h + 13}" rx="8.5" fill="none" stroke="#2c2e30" stroke-width="0.7" opacity="0.6"/>`;
    }
    // hole in the case: static, never moves
    o += `<rect x="${x - 3}" y="${y - 3}" width="${w + 6}" height="${h + 6}" rx="6" fill="url(#hole)" stroke="url(#edgeHole)" stroke-width="1.2"/>`;
    // the cap travels down into the hole when pressed
    return o + `<g class="btn" data-btn="${id}"><g class="capg">` +
      `<rect class="csh" x="${x + 1.2}" y="${y + 2.4}" width="${w}" height="${h}" rx="4" fill="#000" opacity="0.75" filter="url(#blur1)"/>` +
      `<rect class="cap" x="${x}" y="${y}" width="${w}" height="${h}" rx="4" fill="url(#btnGrad)" stroke="#070708" stroke-width="1"/>` +
      `<rect class="sheen" x="${x + 0.8}" y="${y + 0.8}" width="${w - 1.6}" height="${h - 1.6}" rx="3.4" fill="none" stroke="url(#capEdge)" stroke-width="1.4"/>` +
      `<rect class="sheen" x="${x + 3}" y="${y + 1.6}" width="${w - 6}" height="${h * 0.4}" rx="2.5" fill="url(#capTop)"/>` +
      deco + t + '</g></g>';
  }

  // Milled V-groove outline (surface stays flush): the wall facing the top-left light is lit
  function milled(x, y, w, h, r = 3) {
    const R = (dx, stroke, sw, op) =>
      `<rect x="${x + dx}" y="${y + dx}" width="${w}" height="${h}" rx="${r}" fill="none" stroke="${stroke}" stroke-width="${sw}" opacity="${op}"/>`;
    return R(-0.8, '#000', 1.5, 0.55) + R(0.9, '#fff', 1.2, 0.3) + R(0, '#151618', 0.8, 0.9);
  }

  const SLIDERS = [70, 112, 154, 196, 238, 300];
  const SL_TOP = 494, SL_BOT = 610;
  const sliderCapY = (v) => SL_BOT - 14 - (v / 100) * (SL_BOT - SL_TOP - 28); // v: 0..100

  function slider(i, value) {
    const x = SLIDERS[i];
    let o = `<rect x="${x - 6.5}" y="${SL_TOP}" width="13" height="${SL_BOT - SL_TOP}" fill="#070708"/>` +
      `<rect x="${x - 6.5}" y="${SL_TOP}" width="13" height="${SL_BOT - SL_TOP}" fill="none" stroke="url(#edgeHole)" stroke-width="1"/>`;
    for (let k = 0; k <= 10; k++) {
      const y = SL_BOT - 14 - (k / 10) * (SL_BOT - SL_TOP - 28);
      o += `<rect x="${x + 10}" y="${f1(y - 0.4)}" width="${k % 5 === 0 ? 7 : 4.5}" height="0.8" fill="#aeb1b4" opacity="0.8"/>`;
    }
    o += `<g class="slider" data-slider="${i}" transform="translate(0,${f1(sliderCapY(value))})">` +
      `<rect x="${x - 15}" y="-16" width="30" height="32" fill="transparent"/>` +
      `<rect x="${x - 7}" y="-11" width="17" height="27" rx="1.6" fill="#000" opacity="0.45" filter="url(#blur1)"/>` +
      `<rect x="${x - 8}" y="-13.5" width="16" height="27" rx="1.6" fill="#c76f04"/>` +
      `<rect x="${x - 8}" y="-13.5" width="16" height="14" rx="1.6" fill="url(#capGrad)"/>` +
      `<rect x="${x - 8}" y="0.5" width="16" height="11" rx="1" fill="#f39a0c"/>` +
      `<rect x="${x - 8}" y="0" width="16" height="1.3" fill="#ffd27a" opacity="0.9"/>` +
      `<rect x="${x - 8}" y="-13.5" width="16" height="27" rx="1.6" fill="none" stroke="#9c5704" stroke-width="0.6"/>` +
      `</g>`;
    return o;
  }

  const DIAL = [492, 524];

  // Well cut into the case: circle around the knob opening to the right edge
  function wellPath(open = false) {
    const [cx, cy] = DIAL, r = 106;
    // open=true: just the step wall, without the side that runs out through the case edge
    if (open) return `M604,${cy - r} L${cx},${cy - r} A${r},${r} 0 0,0 ${cx},${cy + r} L604,${cy + r}`;
    return `M${cx},${cy - r} L604,${cy - r} L604,${cy + r} L${cx},${cy + r} A${r},${r} 0 0,1 ${cx},${cy - r} Z`;
  }

  // Parts that physically move with rotation (knurl, dots, dimple).
  // Their shading uses unrotated gradients, so the light source stays fixed (top-left).
  function dialKnurl(angle) {
    const [cx, cy] = DIAL;
    return `<circle cx="${cx}" cy="${cy}" r="87.3" fill="none" stroke="#ffb229" stroke-width="3.6" stroke-dasharray="1.6 1.4" transform="rotate(${f1(angle)} ${cx} ${cy})"/>`;
  }
  function dialFace(angle) {
    const [cx, cy] = DIAL;
    const rad = (angle * Math.PI) / 180;
    let o = '';
    for (let i = 0; i < 8; i++) {
      const a = (i * Math.PI) / 4 + 0.2 + rad;
      o += `<circle cx="${f1(cx + Math.cos(a) * 60)}" cy="${f1(cy + Math.sin(a) * 60)}" r="2.3" fill="url(#bump)"/>`;
    }
    const da = Math.atan2(-28, 46) + rad, dr = Math.hypot(46, 28);
    const dx = f1(cx + Math.cos(da) * dr), dy = f1(cy + Math.sin(da) * dr);
    o += `<circle cx="${dx}" cy="${dy}" r="22" fill="url(#dimple)"/>` +
      `<circle cx="${dx}" cy="${dy}" r="21.3" fill="none" stroke="url(#dimpleWall)" stroke-width="1.6"/>`;
    return o;
  }

  // Metallic case paint; reused for lower case levels so they share the same material
  function bodyPaint() {
    const R = (fill, extra = '') => `<rect x="6" y="6" width="598" height="784" rx="24" fill="${fill}" ${extra}/>`;
    return R('url(#body)') + R('url(#paint)') + R('url(#sheen)');
  }

  function dial(angle = 0) {
    const [cx, cy] = DIAL;
    const well = wellPath(), wall = wellPath(true);
    return `<clipPath id="wellClip"><path d="${well}"/></clipPath>` +
      `<g clip-path="url(#wellClip)">` + bodyPaint() +
      `<path d="${well}" fill="#000" opacity="0.06"/>` +
      // inner wall shadow: light from top-left, so the top-left wall shades the floor
      `<path d="${wall}" fill="none" stroke="#000" stroke-width="10" opacity="0.4" filter="url(#blur4)" transform="translate(3,4)"/>` +
      // orange bounce light from the knob on the left wall
      `<path d="M${cx - 40},${cy - 98} A106,106 0 0,0 ${cx - 40},${cy + 98}" fill="none" stroke="#d5780c" stroke-width="7" opacity="0.4" filter="url(#blur2)"/>` +
      // knob shadow on the floor (static)
      `<circle cx="${cx + 7}" cy="${cy + 9}" r="89" fill="#000" opacity="0.55" filter="url(#blur4)"/>` +
      `</g>` +
      `<path d="${wall}" fill="none" stroke="url(#edgeHole)" stroke-width="1.6" clip-path="url(#bodyClip)"/>` +
      // knob side wall (gives height), then rim/knurl
      `<circle cx="${cx + 1.4}" cy="${cy + 2.4}" r="89.5" fill="#a85a00"/>` +
      `<circle cx="${cx}" cy="${cy}" r="89" fill="#d27a03"/>` +
      `<g id="dialKnurl">${dialKnurl(angle)}</g>` +
      `<circle cx="${cx}" cy="${cy}" r="87.3" fill="none" stroke="url(#knurlShade)" stroke-width="4"/>` +
      `<circle cx="${cx}" cy="${cy}" r="84" fill="url(#dialGrad)"/>` +
      `<circle cx="${cx}" cy="${cy}" r="84" fill="none" stroke="url(#edgeLit)" stroke-width="1.4" opacity="0.8"/>` +
      `<g id="dialFace">${dialFace(angle)}</g>` +
      // static specular highlight
      `<ellipse cx="${cx - 26}" cy="${cy - 34}" rx="38" ry="22" transform="rotate(-35 ${cx - 26} ${cy - 34})" fill="#fff" opacity="0.16" filter="url(#blur4)"/>` +
      `<circle id="dialHit" class="dial" cx="${cx}" cy="${cy}" r="92" fill="transparent"/>`;
  }

  function panel(state, assetBase = '') {
    const W = 610, H = 800;
    let o = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" width="${W}" height="${H}" font-family="Arial, Helvetica, 'Liberation Sans', sans-serif" font-weight="bold">`;
    o += `<style>${LCD_CSS}</style>`;
    o += `<defs>
      <linearGradient id="body" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#85888b"/><stop offset="0.5" stop-color="#696c6f"/><stop offset="1" stop-color="#4f5254"/>
      </linearGradient>
      <pattern id="paint" width="80" height="80" patternUnits="userSpaceOnUse"><image href="${assetBase}paint.png" width="80" height="80"/></pattern>
      <linearGradient id="sheen" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#fff" stop-opacity="0.10"/><stop offset="0.28" stop-color="#fff" stop-opacity="0.16"/>
        <stop offset="0.6" stop-color="#fff" stop-opacity="0"/><stop offset="1" stop-color="#fff" stop-opacity="0"/>
      </linearGradient>
      <filter id="blur1"><feGaussianBlur stdDeviation="1"/></filter>
      <filter id="blur2" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="2"/></filter>
      <filter id="blur3"><feGaussianBlur stdDeviation="3"/></filter>
      <filter id="blur4" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="4"/></filter>
      <linearGradient id="hole" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#050506"/><stop offset="1" stop-color="#232427"/></linearGradient>
      <linearGradient id="edgeHole" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#2a2c2e"/><stop offset="0.5" stop-color="#55585b"/><stop offset="1" stop-color="#a7aaad"/></linearGradient>
      <linearGradient id="edgeLit" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ffffff" stop-opacity="0.55"/><stop offset="0.5" stop-color="#ffffff" stop-opacity="0.05"/><stop offset="1" stop-color="#000" stop-opacity="0.45"/></linearGradient>
      <linearGradient id="chamferBig" x1="0" y1="0" x2="0.6" y2="1"><stop offset="0" stop-color="#000" stop-opacity="0.5"/><stop offset="0.45" stop-color="#000" stop-opacity="0.12"/><stop offset="0.7" stop-color="#fff" stop-opacity="0.08"/><stop offset="1" stop-color="#fff" stop-opacity="0.3"/></linearGradient>
      <linearGradient id="bevelV" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.38"/><stop offset="0.08" stop-color="#fff" stop-opacity="0.06"/><stop offset="0.9" stop-color="#000" stop-opacity="0.05"/><stop offset="1" stop-color="#000" stop-opacity="0.5"/></linearGradient>
      <linearGradient id="bevelH" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff" stop-opacity="0.22"/><stop offset="0.1" stop-color="#fff" stop-opacity="0"/><stop offset="0.9" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.38"/></linearGradient>
      <linearGradient id="bezel" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2c2d30"/><stop offset="1" stop-color="#18191b"/></linearGradient>
      <linearGradient id="capEdge" x1="0" y1="0" x2="0.3" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.28"/><stop offset="0.45" stop-color="#fff" stop-opacity="0.04"/><stop offset="1" stop-color="#000" stop-opacity="0.5"/></linearGradient>
      <linearGradient id="capTop" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.13"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>
      <linearGradient id="knurlShade" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.35"/><stop offset="0.45" stop-color="#fff" stop-opacity="0"/><stop offset="1" stop-color="#5a2a00" stop-opacity="0.55"/></linearGradient>
      <radialGradient id="bump" cx="0.35" cy="0.3" r="0.75"><stop offset="0" stop-color="#ffe2a3"/><stop offset="0.5" stop-color="#ffab1e"/><stop offset="1" stop-color="#c46c00"/></radialGradient>
      <linearGradient id="dimpleWall" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#9a5000" stop-opacity="0.8"/><stop offset="0.5" stop-color="#ffa30f" stop-opacity="0"/><stop offset="1" stop-color="#ffe7b0" stop-opacity="0.9"/></linearGradient>
      <filter id="glow" x="-100%" y="-200%" width="300%" height="500%"><feGaussianBlur stdDeviation="3"/></filter>
      <linearGradient id="black" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#151617"/><stop offset="1" stop-color="#0c0c0d"/></linearGradient>
      <linearGradient id="plate" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#606366"/><stop offset="1" stop-color="#717477"/></linearGradient>
      <linearGradient id="btnGrad" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#303134"/><stop offset="0.6" stop-color="#1d1e20"/><stop offset="1" stop-color="#131314"/></linearGradient>
      <linearGradient id="capGrad" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffc24a"/><stop offset="0.5" stop-color="#ffa514"/><stop offset="1" stop-color="#e88e06"/></linearGradient>
      <radialGradient id="dialGrad" cx="0.38" cy="0.33" r="0.8"><stop offset="0" stop-color="#ffbb36"/><stop offset="0.65" stop-color="#ffa310"/><stop offset="1" stop-color="#ec9004"/></radialGradient>
      <radialGradient id="dimple" cx="0.62" cy="0.68" r="0.7"><stop offset="0" stop-color="#ffc85a"/><stop offset="0.55" stop-color="#ffa30f"/><stop offset="1" stop-color="#d77a00"/></radialGradient>
      <radialGradient id="recess" cx="0.5" cy="0.5" r="0.5"><stop offset="0.8" stop-color="#55575a"/><stop offset="1" stop-color="#6c6e71"/></radialGradient>
      <radialGradient id="lcdOff" cx="0.5" cy="0.45" r="0.75"><stop offset="0" stop-color="#bcc3ab"/><stop offset="1" stop-color="#a1a993"/></radialGradient>
      <radialGradient id="lcdOn" cx="0.5" cy="0.45" r="0.75"><stop offset="0" stop-color="#ffc77e"/><stop offset="0.7" stop-color="#f6a24a"/><stop offset="1" stop-color="#df8530"/></radialGradient>
      <linearGradient id="glass" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.16"/><stop offset="0.4" stop-color="#fff" stop-opacity="0.02"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>
    </defs>`;

    // body
    // soft drop shadow under the case
    o += `<rect x="8" y="10" width="594" height="782" rx="24" fill="#000" opacity="0.55" filter="url(#blur3)"/>`;
    o += `<clipPath id="bodyClip"><rect x="6" y="6" width="598" height="784" rx="24"/></clipPath>`;
    o += bodyPaint();

    // black top panel with branding, LEDs and LCD
    o += `<path d="M36,470 V43 a5,5 0 0,1 5,-5 H329 a5,5 0 0,1 5,5 V470 Z" fill="url(#black)"/>`;
    o += `<g transform="translate(185,94)">
        <rect x="-112" y="-20" width="20" height="22" rx="2" fill="none" stroke="#c4c6c8" stroke-width="2.4"/>
        <path d="M-106,-14 v12 h9 v-6 h-9" fill="none" stroke="#c4c6c8" stroke-width="2"/>
        <text x="12" y="1" font-size="34" font-family="'Arial Black', 'DejaVu Sans', sans-serif" font-weight="900" text-anchor="middle" fill="#c4c6c8" letter-spacing="1" transform="scale(1.08,1)">KADENCE</text>
      </g>`;
    o += `<text x="185" y="136" font-size="11.5" text-anchor="middle" fill="#c4c6c8" letter-spacing="0.6">METRONOME <tspan font-style="italic" font-weight="normal">Dr.Tick</tspan></text>`;
    o += `<text x="185" y="180" font-size="44" text-anchor="middle" fill="#c9cbcd" font-family="'Arial Black', 'DejaVu Sans', sans-serif" letter-spacing="2" transform="translate(185,0) scale(1.08,1) translate(-185,0)">KD-90</text>`;
    o += `<rect id="led1" class="led" x="113" y="196" width="20" height="8" rx="2" fill="#3a2a2a"/>`;
    o += `<rect id="led2" class="led" x="237" y="196" width="20" height="8" rx="2" fill="#2a3a2c"/>`;

    // LCD
    o += `<rect x="56" y="214" width="258" height="208" rx="3" fill="#050505" stroke="#2a2a2a"/>`;
    o += `<g id="lcd" transform="translate(60,218)">${lcd(state)}</g>`;
    o += `<rect x="60" y="218" width="250" height="200" fill="url(#glass)" pointer-events="none"/>`;

    // speaker grille
    for (let i = 0; i < 13; i++) {
      const y = 52 + i * 14;
      o += `<rect x="346" y="${y}" width="234" height="7" rx="3.5" fill="#1d1e20"/><rect x="348" y="${y + 7}" width="230" height="1.2" fill="#fff" opacity="0.18"/>`;
    }

    // buttons top-right
    o += button('POWER', 384, 322, 60, 34, 'POWER', 'box', 9.6, true);
    o += button('LIGHT', 458, 322, 60, 34, 'LIGHT');
    o += button('MUTE', 532, 322, 60, 34, 'MUTE');
    o += button('WRITE', 384, 378, 60, 34, 'WRITE', 'plain', 9.6, true);
    o += button('MEMORY', 458, 378, 60, 34, 'MEMORY', 'plain', 8.8);
    o += button('MIDI', 532, 378, 60, 34, 'MIDI');

    // tempo/value/note print
    o += `<g font-size="10" fill="#aeb1b4" opacity="0.9"><text x="352" y="424">TEMPO/</text><text x="352" y="436">VALUE/</text>` +
      `<rect x="351" y="440" width="35" height="12" rx="1" fill="none" stroke="#aeb1b4"/><text x="355" y="450">NOTE</text></g>`;

    // dial
    o += dial();

    // mic
    o += `<text x="368" y="584" font-size="8" fill="#aeb1b4" text-anchor="middle">MIC</text>`;
    for (let i = 0; i < 3; i++) o += `<rect x="${358 + i * 8}" y="590" width="3.6" height="22" rx="1.8" fill="#1d1e20"/>`;

    // slider plate
    // one milled outline around screen panel + slider area; shared grooves split screen / sliders / VOL
    o += milled(36, 38, 298, 588, 5);
    o += `<rect x="36" y="471.9" width="298" height="1.2" fill="#fff" opacity="0.3"/>` +
      `<rect x="36" y="470.6" width="298" height="0.8" fill="#151618" opacity="0.9"/>`;
    o += `<rect x="265.2" y="471" width="1.5" height="155" fill="#000" opacity="0.55"/>` +
      `<rect x="266.9" y="471" width="1.2" height="155" fill="#fff" opacity="0.3"/>` +
      `<rect x="265.6" y="471" width="0.8" height="155" fill="#151618" opacity="0.9"/>`;
    o += `<text x="300" y="489" font-size="8" fill="#aeb1b4" text-anchor="middle">VOL</text>`;
    o += `<g id="sliders">${state.sliders.map((v, i) => slider(i, v)).join('')}</g>`;

    // bottom buttons
    o += button('TEMPO', 75, 670, 60, 36, 'TEMPO');
    o += button('LOOP', 150, 670, 60, 36, 'LOOP');
    o += button('TUNE', 225, 670, 60, 36, 'TUNE', 'box');
    o += button('COACH', 300, 670, 60, 36, 'COACH', 'light');
    o += button('BEAT', 75, 724, 60, 36, 'BEAT/PATTERN', 'plain', 9.4);
    o += button('VOICE', 150, 724, 60, 36, 'VOICE');
    o += button('PITCH', 225, 724, 60, 36, 'PITCH', 'box');
    o += button('TRIG', 300, 724, 60, 36, 'TRIG/GUIDE', 'light', 9.4);
    o += `<path d="M46,746 v6 h24 M154,752 h24 v-6" fill="none" stroke="#aeb1b4" stroke-width="1.2"/><text x="112" y="756" font-size="8.5" fill="#aeb1b4" text-anchor="middle">COUNT IN</text>`;
    o += button('TAP', 380, 697, 54, 90, 'TAP', 'plain', 12);
    o += button('START', 494, 697, 132, 90, 'START/STOP', 'single', 13);

    // rounded case edge (drawn last so it continues past the knob well): same radius all round, lit from top-left, shaded bottom-right
    // full-size rounding on the upper level only; the lowered knob well gets a much smaller edge
    o += `<mask id="upperLevel"><rect x="0" y="0" width="610" height="800" fill="#fff"/><path d="${wellPath()}" fill="#000"/></mask>`;
    o += `<g clip-path="url(#wellClip)"><g clip-path="url(#bodyClip)">` +
      `<rect x="7.2" y="7.2" width="595.6" height="781.6" rx="22.8" fill="none" stroke="url(#bevelH)" stroke-width="2.6" filter="url(#blur1)"/>` +
      `</g></g>`;
    o += `<g clip-path="url(#bodyClip)" mask="url(#upperLevel)">` +
      `<rect x="9.5" y="9.5" width="591" height="777" rx="21" fill="none" stroke="url(#bevelV)" stroke-width="8" filter="url(#blur2)"/>` +
      `<rect x="9.5" y="9.5" width="591" height="777" rx="21" fill="none" stroke="url(#bevelH)" stroke-width="8" filter="url(#blur2)"/>` +
      `<rect x="7" y="7" width="596" height="782" rx="23" fill="none" stroke="#fff" stroke-width="1" opacity="0.12"/>` +
      `</g>`;
    o += `<rect x="6" y="6" width="598" height="784" rx="24" fill="none" stroke="#2e3032" stroke-width="1.2"/>`;
    o += '</svg>';
    return o;
  }

  const LCD_CSS = `
    .btn .capg { transform-box: fill-box; transform-origin: 50% 50%; transition: transform 45ms ease-out, filter 45ms; }
    .btn.down .capg { transform: scale(0.955); filter: brightness(0.8); }
    .btn .csh, .btn .sheen { transition: opacity 45ms; }
    .btn.down .csh { opacity: 0.15; }
    .btn.down .sheen { opacity: 0.35; }
    #lcd .on { fill: #17180f; }
    #lcd .gh { fill: #17180f; opacity: 0.055; }
    #lcd .bl .on { fill: #2b1606; }
    #lcd .bl .gh { fill: #2b1606; opacity: 0.07; }
    #lcd text { font-family: Arial, Helvetica, sans-serif; font-weight: bold; }
    #lcd .lt { fill: #17180f; }
    #lcd .lg { fill: #17180f; opacity: 0; }
    #lcd .bl .lt { fill: #2b1606; }
    #lcd .inv { fill: #b4bba2; }
    #lcd .bl .inv { fill: #f6a24a; }
  `;

  const api = { panel, lcd, dialKnurl, dialFace, sliderCapY, SLIDERS, SL_TOP, SL_BOT, DIAL, PEND_N, LCD_CSS };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  else root.KD90 = api;
})(this);
