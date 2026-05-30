/* core.js — runs on every page. Imports ECharts theme from charts.js.
 *
 *  - Ambient constellation (drift)
 *  - Count-up tickers on stat values        (data-ticker="<number>")
 *  - Marquee on overflow text               (data-marquee on the element)
 *  - Click ripple on .btn / .role-apply
 *  - Apply-button window.open before HTMX intercepts the click
 *  - pane-mounted class triggers width-based transitions
 */

(function () {
  'use strict';

  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // ── Ambient constellation ──────────────────────────────────────
  function startAmbient() {
    if (reduceMotion) return;
    const canvas = document.createElement('canvas');
    canvas.id = 'ambient-canvas';
    document.body.appendChild(canvas);
    const ctx = canvas.getContext('2d');
    let w, h, dots;
    const N = 56;
    function resize() {
      w = canvas.width = window.innerWidth;
      h = canvas.height = window.innerHeight;
      dots = Array.from({ length: N }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * 0.12,
        vy: (Math.random() - 0.5) * 0.12,
        r: Math.random() * 1.6 + 0.7,
        a: Math.random() * 0.5 + 0.22,
      }));
    }
    resize();
    window.addEventListener('resize', resize);
    function tick() {
      ctx.clearRect(0, 0, w, h);
      for (const d of dots) {
        d.x += d.vx; d.y += d.vy;
        if (d.x < 0) d.x = w; if (d.x > w) d.x = 0;
        if (d.y < 0) d.y = h; if (d.y > h) d.y = 0;
        ctx.fillStyle = `rgba(126, 168, 255, ${d.a})`;
        ctx.beginPath();
        ctx.arc(d.x, d.y, d.r, 0, Math.PI * 2);
        ctx.fill();
      }
      requestAnimationFrame(tick);
    }
    tick();
  }

  // ── Count-up tickers ───────────────────────────────────────────
  function formatNum(n) { return n >= 10000 ? n.toLocaleString() : String(n); }
  function tickUp(el, target, durationMs) {
    if (reduceMotion) { el.textContent = formatNum(target); return; }
    const start = performance.now();
    function frame(now) {
      const t = Math.min(1, (now - start) / durationMs);
      const eased = 1 - Math.pow(1 - t, 3);
      el.textContent = formatNum(Math.round(target * eased));
      if (t < 1) requestAnimationFrame(frame);
      else el.textContent = formatNum(target);
    }
    requestAnimationFrame(frame);
  }
  function bootTickers() {
    document.querySelectorAll('[data-ticker]').forEach((el) => {
      const target = parseInt(el.getAttribute('data-ticker'), 10);
      if (!isNaN(target)) tickUp(el, target, 700);
    });
  }

  // ── Marquee on overflow ────────────────────────────────────────
  function applyMarquee() {
    document.querySelectorAll('[data-marquee]').forEach((el) => {
      if (el.querySelector('.marquee-inner')) return;
      const containerW = el.clientWidth;
      // Skip if the container hasn't been laid out yet (width ~0). The
      // window-load deferred run will catch it once layout completes;
      // running anyway would marquee everything with a huge shift and
      // hide the real text off-screen.
      if (containerW < 50) return;
      const text = el.textContent;
      const probe = document.createElement('span');
      probe.style.cssText = 'position:absolute;visibility:hidden;white-space:nowrap;font:inherit;';
      probe.textContent = text;
      document.body.appendChild(probe);
      const textW = probe.scrollWidth;
      probe.remove();
      // Only marquee if the overflow is real (>12px), not a 1-2px subpixel slip.
      if (textW > containerW + 12) {
        el.classList.add('marquee');
        const inner = document.createElement('span');
        inner.className = 'marquee-inner';
        inner.textContent = text;
        const shift = (textW - containerW + 32);
        inner.style.setProperty('--marquee-shift', shift + 'px');
        el.innerHTML = '';
        el.appendChild(inner);
      }
    });
  }

  // ── Click ripple ───────────────────────────────────────────────
  function attachRipple() {
    document.addEventListener('click', (ev) => {
      const btn = ev.target.closest('.btn, .role-apply');
      if (!btn) return;
      const rect = btn.getBoundingClientRect();
      btn.style.setProperty('--ripple-x', (ev.clientX - rect.left) + 'px');
      btn.style.setProperty('--ripple-y', (ev.clientY - rect.top) + 'px');
      btn.classList.remove('rippling');
      void btn.offsetWidth;
      btn.classList.add('rippling');
      setTimeout(() => btn.classList.remove('rippling'), 500);
    });
  }

  // ── Apply: force window.open BEFORE HTMX intercepts. ──────────
  // Captured during the capture phase so it runs before HTMX's own
  // bubble-phase click handler (which prevents the default anchor
  // navigation when hx-post is present).
  function attachApplyOpener() {
    document.addEventListener('click', (ev) => {
      const a = ev.target.closest('a.role-apply, a.btn-apply');
      if (!a) return;
      const href = a.getAttribute('href');
      if (href && href !== '#') {
        try { window.open(href, '_blank', 'noopener'); } catch (_) {}
      }
    }, true);
  }

  // ── Pane mount class — drives width-based bar transitions ─────
  function mountPanes() {
    requestAnimationFrame(() => {
      document.querySelectorAll('.panel, .pane').forEach((p) => p.classList.add('pane-mounted'));
    });
  }

  function init() {
    startAmbient();
    bootTickers();
    attachRipple();
    attachApplyOpener();
    mountPanes();
    // Marquee detection needs layout to be settled. Run once on full load
    // so element widths are accurate; running on DOMContentLoaded reads
    // pre-layout zero-widths and breaks every marqueed element.
    if (document.readyState === 'complete') {
      applyMarquee();
    } else {
      window.addEventListener('load', applyMarquee);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
