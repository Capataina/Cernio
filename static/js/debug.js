/* debug.js — wires the floating "snap all" button to POST /debug/snap-all
 * and renders the result as a one-click-copyable path toast.
 */
(function () {
  'use strict';

  function init() {
    const btn = document.getElementById('snap-all');
    const toast = document.getElementById('snap-toast');
    if (!btn || !toast) return;

    btn.addEventListener('click', async () => {
      btn.setAttribute('data-state', 'running');
      btn.querySelector('.snap-label').textContent = 'snapping…';
      showToast({ head: 'capturing 4 tabs', body: 'headless chrome is rendering each page…', meta: '' }, false);

      try {
        const resp = await fetch('/debug/snap-all', { method: 'POST' });
        const json = await resp.json();
        if (json.ok) {
          const files = (json.files || []).map((f) => f.split('/').pop()).join(', ');
          showToast({
            head: `captured ${json.files ? json.files.length : 0} pages · ${json.elapsed_ms}ms`,
            body: json.folder,
            meta: files,
          }, false);
        } else {
          showToast({
            head: 'snap failed',
            body: json.error || 'unknown error',
            meta: `${json.elapsed_ms || '?'}ms`,
          }, true);
        }
      } catch (e) {
        showToast({ head: 'snap failed', body: String(e), meta: '' }, true);
      } finally {
        btn.removeAttribute('data-state');
        btn.querySelector('.snap-label').textContent = 'snap all';
      }
    });
  }

  function showToast({ head, body, meta }, isError) {
    const toast = document.getElementById('snap-toast');
    if (!toast) return;
    toast.innerHTML =
      `<div class="snap-toast-head">${escape(head)}</div>` +
      `<div class="snap-toast-path">${escape(body)}</div>` +
      (meta ? `<div class="snap-toast-meta">${escape(meta)}</div>` : '');
    toast.classList.toggle('error', !!isError);
    toast.classList.add('visible');
    // Auto-hide after 18s unless errored
    if (!isError) {
      clearTimeout(showToast._t);
      showToast._t = setTimeout(() => toast.classList.remove('visible'), 18000);
    }
  }

  function escape(s) {
    return String(s)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
