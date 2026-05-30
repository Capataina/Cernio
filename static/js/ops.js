// ops.js — ops menu wiring.
//
// Behaviour:
//   - Click #ops-btn → toggle #ops-panel.
//   - Clicking outside the panel (and not the button) closes it.
//   - On open: GET /ops/<op>/preview for each item, populate .ops-preview
//     with summary and stash the JSON detail on the element.
//   - Hovering an item shows the detail JSON in its .ops-detail block.
//   - Clicking .ops-run confirms via confirm(), POSTs /ops/<op>/run
//     (with ?scope=… for unarchive), and toasts the result via #snap-toast.

(function () {
  document.addEventListener('DOMContentLoaded', init);

  const OPS = ['clean', 'format', 'check', 'search', 'unarchive'];

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

    // Hover detail toggling.
    document.querySelectorAll('.ops-item').forEach((item) => {
      const op = item.getAttribute('data-op');
      const detail = item.querySelector('[data-op-detail]');
      item.addEventListener('mouseenter', () => {
        if (detail && detail.textContent.trim()) {
          detail.classList.remove('hidden');
        }
      });
      item.addEventListener('mouseleave', () => {
        if (detail) detail.classList.add('hidden');
      });
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

  function loadPreview(op) {
    const previewEl = document.querySelector(`[data-op-preview="${op}"]`);
    const detailEl = document.querySelector(`[data-op-detail="${op}"]`);
    if (!previewEl) return;
    let url = `/ops/${op}/preview`;
    if (op === 'unarchive') url += `?scope=${currentScope()}`;
    previewEl.textContent = 'loading…';
    fetch(url)
      .then((r) => r.json())
      .then((j) => {
        if (j.ok === false) {
          previewEl.textContent = 'error';
          previewEl.title = j.error || 'preview failed';
        } else {
          previewEl.textContent = j.summary || '—';
          previewEl.title = j.summary || '';
          if (detailEl) {
            detailEl.textContent = JSON.stringify(j.detail || {}, null, 2);
          }
        }
      })
      .catch((e) => {
        previewEl.textContent = 'error';
        previewEl.title = String(e);
      });
  }

  function currentScope() {
    const checked = document.querySelector('input[name="unarchive-scope"]:checked');
    return checked ? checked.value : 'jobs';
  }

  function runOp(op, item, runBtn) {
    if (op === 'search') {
      toast({ head: 'search', body: 'search must be invoked via `cernio search` (CLI).' }, true);
      return;
    }
    const previewText = item.querySelector('[data-op-preview]').textContent || '';
    const label = item.querySelector('h4').textContent || op;
    const scope = op === 'unarchive' ? currentScope() : null;
    const prompt = scope
      ? `Run ${label} (scope=${scope})? Preview: ${previewText}`
      : `Run ${label}? Preview: ${previewText}`;
    if (!window.confirm(prompt)) return;

    let url = `/ops/${op}/run`;
    if (scope) url += `?scope=${scope}`;

    runBtn.disabled = true;
    const original = runBtn.textContent;
    runBtn.textContent = 'running…';
    toast({ head: label.toLowerCase(), body: 'running…' }, false);

    fetch(url, { method: 'POST' })
      .then((r) => r.json())
      .then((j) => {
        runBtn.disabled = false;
        runBtn.textContent = original;
        if (j.ok === false) {
          toast({
            head: `${label} failed`,
            body: j.error || 'unknown error',
            meta: `${j.elapsed_ms || 0} ms`,
          }, true);
        } else {
          toast({
            head: label.toLowerCase(),
            body: j.summary || 'done',
            meta: `${j.elapsed_ms || 0} ms`,
          }, false);
          // Refresh this op's preview after a successful run.
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

  function esc(s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }
})();
