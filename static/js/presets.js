/* presets.js — Saved-searches dropdown using localStorage.
 *
 * Lives top-right in the chrome. Lets the user name + bookmark any
 * filter URL on /jobs, /companies, /decisions, /activity and recall it
 * with a click. Star-icon button opens the panel; per-item delete.
 *
 * Storage shape:
 *   localStorage.cernio_presets = JSON.stringify([
 *     { name: "Untouched SS HFT", path: "/jobs?grade=SS&lane=hft&decision=none", at: 1716... },
 *     ...
 *   ])
 */
(function () {
  'use strict';
  document.addEventListener('DOMContentLoaded', init);

  const STORE_KEY = 'cernio_presets';

  function load() {
    try { return JSON.parse(localStorage.getItem(STORE_KEY) || '[]'); }
    catch (_) { return []; }
  }
  function save(list) {
    try { localStorage.setItem(STORE_KEY, JSON.stringify(list)); }
    catch (_) {}
  }

  function init() {
    const btn = document.getElementById('preset-btn');
    const panel = document.getElementById('preset-panel');
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
  }

  function openPanel() {
    const btn = document.getElementById('preset-btn');
    const panel = document.getElementById('preset-panel');
    btn.setAttribute('data-open', '1');
    panel.classList.remove('hidden');
    render();
    const input = panel.querySelector('input.preset-name-input');
    if (input) input.focus();
  }
  function closePanel() {
    const btn = document.getElementById('preset-btn');
    const panel = document.getElementById('preset-panel');
    btn.removeAttribute('data-open');
    panel.classList.add('hidden');
  }

  function render() {
    const panel = document.getElementById('preset-panel');
    const presets = load();
    const currentPath = location.pathname + location.search;
    const onFilterPage = /^\/(jobs|companies|decisions|activity)/.test(location.pathname);
    const hasFilter = location.search.length > 1;

    let html = '<div class="preset-section-head">Saved searches</div>';
    if (!presets.length) {
      html += '<div class="preset-empty">No saved searches yet.</div>';
    } else {
      for (const [i, p] of presets.entries()) {
        const tag = tagFromPath(p.path);
        html += `<div class="preset-item">
          <a class="preset-name" href="${esc(p.path)}" data-go="${i}">${esc(p.name)}</a>
          <span class="preset-page-tag">${esc(tag)}</span>
          <button class="preset-del" data-del="${i}" title="delete">×</button>
        </div>`;
      }
    }
    if (onFilterPage && hasFilter) {
      html += `<div class="preset-save-row">
        <input class="preset-name-input" type="text" placeholder="Name this search…" autocomplete="off" maxlength="48"/>
        <button class="preset-save-btn">Save</button>
      </div>`;
    } else if (onFilterPage) {
      html += '<div class="preset-empty">Apply some filters first, then come back here to save.</div>';
    } else {
      html += '<div class="preset-empty">Go to Jobs / Companies / Decisions to save searches.</div>';
    }
    panel.innerHTML = html;

    panel.querySelectorAll('[data-del]').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        const idx = parseInt(btn.getAttribute('data-del'), 10);
        const list = load();
        list.splice(idx, 1);
        save(list);
        render();
      });
    });
    const saveBtn = panel.querySelector('.preset-save-btn');
    const nameInput = panel.querySelector('.preset-name-input');
    if (saveBtn && nameInput) {
      const doSave = () => {
        const name = nameInput.value.trim();
        if (!name) { nameInput.focus(); return; }
        const list = load();
        list.unshift({ name, path: currentPath, at: Date.now() });
        save(list.slice(0, 30));
        render();
      };
      saveBtn.addEventListener('click', doSave);
      nameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') { e.preventDefault(); doSave(); }
      });
    }
  }

  function tagFromPath(path) {
    if (path.startsWith('/jobs')) return 'jobs';
    if (path.startsWith('/companies')) return 'cos';
    if (path.startsWith('/decisions')) return 'dec';
    if (path.startsWith('/activity')) return 'act';
    return '·';
  }

  function esc(s) {
    return String(s)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }
})();
