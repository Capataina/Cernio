// ops.js — ops menu wiring (Clean DB + Format text only).
//
// Behaviour:
//   - Click #ops-btn → toggle #ops-panel.
//   - Clicking outside the panel closes it. Esc closes it too.
//   - On open: GET /ops/<op>/preview for each item, populate summary +
//     structured detail (rendered as a small breakdown grid, not raw JSON).
//   - Clicking .ops-run confirms via confirm(), POSTs /ops/<op>/run, and
//     toasts the result via #snap-toast. Successful run refreshes preview.

(function () {
  document.addEventListener('DOMContentLoaded', init);

  const OPS = ['clean', 'format'];

  function init() {
    const btn = document.getElementById('ops-btn');
    const panel = document.getElementById('ops-panel');
    if (!btn || !panel) return;

    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const open = !panel.classList.contains('hidden');
      if (open) closePanel(); else openPanel();
    });

    document.addEventListener('click', (e) => {
      if (panel.classList.contains('hidden')) return;
      if (panel.contains(e.target) || btn.contains(e.target)) return;
      closePanel();
    });
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && !panel.classList.contains('hidden')) closePanel();
    });

    document.querySelectorAll('.ops-item').forEach((item) => {
      const op = item.getAttribute('data-op');
      const runBtn = item.querySelector('[data-op-run]');
      if (runBtn) {
        runBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          runOp(op, item, runBtn);
        });
      }
    });
  }

  function openPanel() {
    const btn = document.getElementById('ops-btn');
    const panel = document.getElementById('ops-panel');
    panel.classList.remove('hidden');
    btn.setAttribute('data-open', '1');
    OPS.forEach(loadPreview);
  }
  function closePanel() {
    const btn = document.getElementById('ops-btn');
    const panel = document.getElementById('ops-panel');
    panel.classList.add('hidden');
    btn.removeAttribute('data-open');
  }

  // ── preview load + render ────────────────────────────────────────

  function loadPreview(op) {
    const previewEl = document.querySelector(`[data-op-preview="${op}"]`);
    const detailEl = document.querySelector(`[data-op-detail="${op}"]`);
    if (!previewEl) return;
    previewEl.textContent = 'loading…';
    if (detailEl) detailEl.innerHTML = '<span class="ops-detail-loading">loading preview…</span>';

    fetch(`/ops/${op}/preview`)
      .then((r) => r.json())
      .then((j) => {
        if (j.ok === false) {
          previewEl.textContent = 'error';
          previewEl.title = j.error || 'preview failed';
          if (detailEl) detailEl.innerHTML = `<div class="ops-error">${esc(j.error || 'preview failed')}</div>`;
          return;
        }
        previewEl.textContent = j.summary || '—';
        previewEl.title = j.summary || '';
        if (detailEl) detailEl.innerHTML = renderDetail(op, j.detail || {});
      })
      .catch((e) => {
        previewEl.textContent = 'error';
        previewEl.title = String(e);
      });
  }

  // Render the structured preview detail per-op as a chip-style breakdown
  // (instead of raw JSON dump). Returns innerHTML string.
  function renderDetail(op, d) {
    if (op === 'clean') return renderCleanDetail(d);
    if (op === 'format') return renderFormatDetail(d);
    return renderGeneric(d);
  }

  function renderCleanDetail(d) {
    const parts = [];
    // Headline counts
    parts.push(`<div class="ops-detail-headline">
      <span class="ops-detail-num">${fmt(d.would_archive_total || 0)}</span>
      <span class="ops-detail-num-label">would be archived</span>
    </div>`);

    if (d.by_grade && typeof d.by_grade === 'object') {
      const rows = Object.entries(d.by_grade)
        .filter(([, n]) => n > 0)
        .sort((a, b) => GRADE_ORDER.indexOf(a[0]) - GRADE_ORDER.indexOf(b[0]))
        .map(([g, n]) => `<span class="ops-chip ops-chip-grade ops-chip-grade-${g.toLowerCase()}">${esc(g)} <b>${fmt(n)}</b></span>`)
        .join('');
      if (rows) parts.push(`<div class="ops-detail-row"><span class="ops-detail-label">By grade</span><div class="ops-chips">${rows}</div></div>`);
    }
    if (d.by_lane && typeof d.by_lane === 'object') {
      const rows = Object.entries(d.by_lane)
        .filter(([, n]) => n > 0)
        .sort((a, b) => b[1] - a[1])
        .map(([l, n]) => `<span class="ops-chip ops-chip-lane" style="--lane-color: ${laneColor(l)}">${esc(laneLabel(l))} <b>${fmt(n)}</b></span>`)
        .join('');
      if (rows) parts.push(`<div class="ops-detail-row"><span class="ops-detail-label">By lane</span><div class="ops-chips">${rows}</div></div>`);
    }
    // Secondary metrics rendered as small kv cards
    const kv = [];
    if ('archived_to_purge' in d)   kv.push(['Archived → purge', d.archived_to_purge]);
    if ('low_grade_companies' in d) kv.push(['Low-grade companies', d.low_grade_companies]);
    if ('orphan_decisions' in d)    kv.push(['Orphan decisions', d.orphan_decisions]);
    if (kv.length) {
      parts.push(`<div class="ops-detail-row"><div class="ops-kv-grid">${
        kv.map(([k, v]) => `<div class="ops-kv"><span class="ops-kv-label">${esc(k)}</span><span class="ops-kv-val">${fmt(v || 0)}</span></div>`).join('')
      }</div></div>`);
    }
    return parts.join('') || renderGeneric(d);
  }

  function renderFormatDetail(d) {
    const parts = [];
    parts.push(`<div class="ops-detail-headline">
      <span class="ops-detail-num">${fmt(d.candidates || 0)}</span>
      <span class="ops-detail-num-label">candidates · capped at ${fmt(d.cap || 50)}</span>
    </div>`);
    const kv = [];
    if ('jobs_descriptions' in d) kv.push(['Job descriptions', d.jobs_descriptions]);
    if ('jobs_assessments' in d)  kv.push(['Fit assessments', d.jobs_assessments]);
    if ('companies_descriptions' in d) kv.push(['Company descriptions', d.companies_descriptions]);
    if ('would_mutate' in d) kv.push(['Would mutate', d.would_mutate]);
    if (kv.length) {
      parts.push(`<div class="ops-detail-row"><div class="ops-kv-grid">${
        kv.map(([k, v]) => `<div class="ops-kv"><span class="ops-kv-label">${esc(k)}</span><span class="ops-kv-val">${fmt(v || 0)}</span></div>`).join('')
      }</div></div>`);
    }
    return parts.join('') || renderGeneric(d);
  }

  function renderGeneric(d) {
    if (!d || typeof d !== 'object') return '';
    const rows = Object.entries(d).map(([k, v]) => {
      const valStr = (typeof v === 'object') ? JSON.stringify(v) : String(v);
      return `<div class="ops-kv"><span class="ops-kv-label">${esc(humanise(k))}</span><span class="ops-kv-val">${esc(valStr)}</span></div>`;
    }).join('');
    return `<div class="ops-detail-row"><div class="ops-kv-grid">${rows}</div></div>`;
  }

  // ── run ──────────────────────────────────────────────────────────

  function runOp(op, item, runBtn) {
    const previewText = item.querySelector('[data-op-preview]').textContent || '';
    const label = item.querySelector('h4').textContent || op;
    if (!window.confirm(`Run ${label}? Preview: ${previewText}`)) return;

    runBtn.disabled = true;
    const original = runBtn.textContent;
    runBtn.textContent = 'running…';
    toast({ head: label.toLowerCase(), body: 'running…' }, false);

    fetch(`/ops/${op}/run`, { method: 'POST' })
      .then((r) => r.json())
      .then((j) => {
        runBtn.disabled = false;
        runBtn.textContent = original;
        if (j.ok === false) {
          toast({ head: `${label} failed`, body: j.error || 'unknown error', meta: `${j.elapsed_ms || 0} ms` }, true);
        } else {
          toast({ head: label.toLowerCase(), body: j.summary || 'done', meta: `${j.elapsed_ms || 0} ms` }, false);
          loadPreview(op);
        }
      })
      .catch((e) => {
        runBtn.disabled = false;
        runBtn.textContent = original;
        toast({ head: `${label} failed`, body: String(e) }, true);
      });
  }

  function toast({ head, body, meta }, isError) {
    const el = document.getElementById('snap-toast');
    if (!el) return;
    el.classList.toggle('error', !!isError);
    el.innerHTML =
      `<div class="snap-toast-head">${esc(head || '')}</div>` +
      `<div class="snap-toast-path">${esc(body || '')}</div>` +
      (meta ? `<div class="snap-toast-meta">${esc(meta)}</div>` : '');
    el.classList.add('visible');
    if (toast._t) clearTimeout(toast._t);
    toast._t = setTimeout(() => el.classList.remove('visible'), 12000);
  }

  // ── helpers ──────────────────────────────────────────────────────

  const GRADE_ORDER = ['SS', 'S', 'A', 'B', 'C', 'F', '—'];

  const LANE_HEX = {
    'big-tech': '#4f7cff', 'ai-ml': '#b66cf0', 'hft': '#ff5c5c',
    'crypto-mm': '#ffc94a', 'bank-strats': '#3acfe5', 'systems-infra': '#52d678',
    'devtools': '#7ea8ff', 'fintech': '#7adf9a',
  };
  const LANE_LABEL = {
    'big-tech': 'Big Tech', 'ai-ml': 'AI / ML', 'hft': 'HFT', 'crypto-mm': 'Crypto MM',
    'bank-strats': 'Bank Strats', 'systems-infra': 'Sys Infra',
    'devtools': 'Devtools', 'fintech': 'Fintech',
  };
  function laneColor(k) { return LANE_HEX[k] || '#7a838f'; }
  function laneLabel(k) { return LANE_LABEL[k] || k; }
  function humanise(k) { return k.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()); }
  function fmt(n) {
    n = Number(n) || 0;
    return n >= 10000 ? n.toLocaleString() : String(n);
  }
  function esc(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }
})();
