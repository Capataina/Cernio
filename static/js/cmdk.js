// Cmd-K palette + global keyboard shortcuts.
// Pure client-side: fetches /api/search-index.json once, filters in memory.

(function () {
    'use strict';

    // ─── Lane colour map (mirrors src/data/lane.rs::lane_hex). ─────────────
    const LANE_HEX = {
        'hft': '#ff6b6b',
        'ai-ml': '#b48cff',
        'systems': '#8cc8ff',
        'gamedev': '#ffb86b',
        'web3': '#ffd764',
        'devtools': '#82d8c8',
        'fintech': '#7adf9a',
    };

    // ─── Built-in commands. ─────────────────────────────────────────────────
    const COMMANDS = [
        { name: 'Dashboard',                    url: '/',                              keywords: 'dashboard home' },
        { name: 'Jobs',                         url: '/jobs',                          keywords: 'jobs roles listings' },
        { name: 'Companies',                    url: '/companies',                     keywords: 'companies orgs' },
        { name: 'Activity',                     url: '/activity',                      keywords: 'activity events log' },
        { name: 'Decisions',                    url: '/decisions',                     keywords: 'decisions' },
        { name: 'Untouched SS jobs',            url: '/jobs?grade=SS&decision=none',   keywords: 'untouched ss top' },
        { name: 'Untouched S jobs',             url: '/jobs?grade=S&decision=none',    keywords: 'untouched s' },
        { name: 'Watching',                     url: '/jobs?decision=watching',        keywords: 'watching watch' },
        { name: 'Applied',                      url: '/jobs?decision=applied',         keywords: 'applied' },
        { name: 'Clean DB (use ops menu)',      url: null,                             keywords: 'clean db ops cleanup', hint: 'Open the ⚙ ops panel to run clean' },
    ];

    // ─── State. ─────────────────────────────────────────────────────────────
    let index = { companies: [], jobs: [] };
    let indexFetchedAt = 0;
    const INDEX_TTL_MS = 5 * 60 * 1000;
    let results = [];
    let selectedIdx = 0;
    let debounceTimer = null;

    // Leader-mode state.
    let leaderActive = false;
    let leaderTimer = null;

    // ─── DOM. ───────────────────────────────────────────────────────────────
    const $palette  = () => document.getElementById('cmdk-palette');
    const $backdrop = () => document.getElementById('cmdk-backdrop');
    const $input    = () => document.getElementById('cmdk-input');
    const $results  = () => document.getElementById('cmdk-results');
    const $leader   = () => document.getElementById('cmdk-leader');

    // ─── Index fetch / cache. ───────────────────────────────────────────────
    async function ensureIndex() {
        const now = Date.now();
        if (now - indexFetchedAt < INDEX_TTL_MS && (index.companies.length || index.jobs.length)) {
            return;
        }
        try {
            const res = await fetch('/api/search-index.json');
            if (!res.ok) return;
            const data = await res.json();
            index = {
                companies: Array.isArray(data.companies) ? data.companies : [],
                jobs:      Array.isArray(data.jobs)      ? data.jobs      : [],
            };
            indexFetchedAt = now;
        } catch (_) { /* offline / 404 — palette still works for commands */ }
    }

    // ─── Ranking. ───────────────────────────────────────────────────────────
    // Exact > prefix > substring; tie-break by grade rank.
    const GRADE_RANK = { 'SS': 0, 'S': 1, 'A': 2, 'B': 3, 'C': 4, 'F': 5 };

    function scoreEntry(entry, q) {
        const name = (entry.name || '').toLowerCase();
        if (!q) return null;
        if (name === q) return 0;
        if (name.startsWith(q)) return 1;
        const idx = name.indexOf(q);
        if (idx >= 0) return 2 + idx * 0.001;
        return null;
    }

    function gradeScore(g) {
        return GRADE_RANK[g] ?? 6;
    }

    function rank(q) {
        const ql = q.trim().toLowerCase();

        // Command mode: query starts with ">".
        if (ql.startsWith('>')) {
            const sub = ql.slice(1).trim();
            const cmds = COMMANDS
                .map(c => {
                    const haystack = (c.name + ' ' + (c.keywords || '')).toLowerCase();
                    if (!sub) return { entry: { ...c, kind: 'cmd' }, score: 1 };
                    if (haystack.includes(sub)) {
                        const idx = haystack.indexOf(sub);
                        return { entry: { ...c, kind: 'cmd' }, score: idx };
                    }
                    return null;
                })
                .filter(Boolean)
                .sort((a, b) => a.score - b.score)
                .slice(0, 30)
                .map(r => r.entry);
            return cmds;
        }

        if (!ql) return [];

        const merged = [];
        for (const e of index.companies) {
            const s = scoreEntry(e, ql);
            if (s !== null) merged.push({ entry: e, score: s });
        }
        for (const e of index.jobs) {
            const s = scoreEntry(e, ql);
            if (s !== null) merged.push({ entry: e, score: s });
        }
        merged.sort((a, b) => {
            if (a.score !== b.score) return a.score - b.score;
            return gradeScore(a.entry.grade) - gradeScore(b.entry.grade);
        });
        return merged.slice(0, 30).map(r => r.entry);
    }

    // ─── Render. ────────────────────────────────────────────────────────────
    function highlight(name, q) {
        if (!q || q.startsWith('>')) return escapeHtml(name);
        const ql = q.toLowerCase();
        const nameLower = name.toLowerCase();
        const idx = nameLower.indexOf(ql);
        if (idx < 0) return escapeHtml(name);
        return escapeHtml(name.slice(0, idx)) +
               '<mark>' + escapeHtml(name.slice(idx, idx + ql.length)) + '</mark>' +
               escapeHtml(name.slice(idx + ql.length));
    }

    function escapeHtml(s) {
        return String(s)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;');
    }

    function render(q) {
        const root = $results();
        if (!root) return;
        if (!results.length) {
            root.innerHTML = q
                ? '<div class="cmdk-empty">No matches. Try ">" for commands.</div>'
                : '<div class="cmdk-empty">Type to search. ">" lists commands.</div>';
            return;
        }
        const parts = [];
        results.forEach((e, i) => {
            const isCmd = e.kind === 'cmd';
            const kindLabel = isCmd ? 'CMD' : (e.kind === 'co' ? 'CO' : 'JOB');
            const kindClass = isCmd ? 'kind-cmd' : (e.kind === 'co' ? 'kind-co' : 'kind-job');
            const sel = i === selectedIdx ? ' is-selected' : '';
            const laneKey = e.lane;
            const laneColor = laneKey ? (LANE_HEX[laneKey] || '#888') : null;
            const laneHtml = laneKey
                ? `<span class="cmdk-lane" style="--lane-color:${laneColor}">${escapeHtml(laneKey)}</span>`
                : '';
            const gradeHtml = e.grade
                ? `<span class="cmdk-grade g-${escapeHtml(e.grade)}">${escapeHtml(e.grade)}</span>`
                : '';
            const tail = isCmd ? '<span class="cmdk-arrow">↗</span>' : '';
            parts.push(
                `<div class="cmdk-row${sel}" data-idx="${i}">` +
                    `<span class="cmdk-kind ${kindClass}">${kindLabel}</span>` +
                    `<span class="cmdk-name">${highlight(e.name, q)}</span>` +
                    gradeHtml + laneHtml + tail +
                `</div>`
            );
        });
        root.innerHTML = parts.join('');
        // Scroll selected into view.
        const selEl = root.querySelector('.cmdk-row.is-selected');
        if (selEl) selEl.scrollIntoView({ block: 'nearest' });
    }

    // ─── Open / close. ──────────────────────────────────────────────────────
    function isOpen() {
        const p = $palette();
        return p && !p.classList.contains('hidden');
    }

    function open(initial) {
        ensureIndex();
        const p = $palette(); const b = $backdrop(); const inp = $input();
        if (!p || !b || !inp) return;
        p.classList.remove('hidden');
        b.classList.remove('hidden');
        inp.value = initial || '';
        selectedIdx = 0;
        results = rank(inp.value);
        render(inp.value);
        setTimeout(() => inp.focus(), 0);
    }

    function close() {
        const p = $palette(); const b = $backdrop();
        if (p) p.classList.add('hidden');
        if (b) b.classList.add('hidden');
    }

    function activate(entry, newTab) {
        if (!entry || !entry.url) {
            // Command with no url (e.g. clean db hint) — close silently.
            close();
            return;
        }
        if (newTab) {
            window.open(entry.url, '_blank', 'noopener');
        } else {
            window.location.href = entry.url;
        }
    }

    // ─── Input handling. ────────────────────────────────────────────────────
    function onInput(e) {
        clearTimeout(debounceTimer);
        const q = e.target.value;
        debounceTimer = setTimeout(() => {
            results = rank(q);
            selectedIdx = 0;
            render(q);
        }, 30);
    }

    function onKeydownInPalette(e) {
        if (e.key === 'Escape') {
            e.preventDefault();
            close();
            return;
        }
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            if (results.length) {
                selectedIdx = (selectedIdx + 1) % results.length;
                render($input().value);
            }
            return;
        }
        if (e.key === 'ArrowUp') {
            e.preventDefault();
            if (results.length) {
                selectedIdx = (selectedIdx - 1 + results.length) % results.length;
                render($input().value);
            }
            return;
        }
        if (e.key === 'Enter') {
            e.preventDefault();
            const newTab = e.metaKey || e.ctrlKey;
            const entry = results[selectedIdx];
            if (entry) activate(entry, newTab);
            return;
        }
    }

    function onResultsClick(e) {
        const row = e.target.closest('.cmdk-row');
        if (!row) return;
        const idx = parseInt(row.dataset.idx, 10);
        const entry = results[idx];
        if (entry) activate(entry, e.metaKey || e.ctrlKey);
    }

    function onBackdropClick() { close(); }

    // ─── Global shortcuts. ──────────────────────────────────────────────────
    function isTypingTarget(el) {
        if (!el) return false;
        const tag = el.tagName;
        if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
        if (el.isContentEditable) return true;
        return false;
    }

    function showLeader() {
        const l = $leader();
        if (l) l.classList.remove('hidden');
    }
    function hideLeader() {
        leaderActive = false;
        const l = $leader();
        if (l) l.classList.add('hidden');
        clearTimeout(leaderTimer);
    }

    function go(url) { window.location.href = url; }

    function onGlobalKeydown(e) {
        // Cmd-K / Ctrl-K — open palette.
        if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
            e.preventDefault();
            if (isOpen()) close(); else open('');
            return;
        }

        // Esc — close palette (handled in palette), drawer, ops panel.
        if (e.key === 'Escape') {
            if (isOpen()) { close(); return; }
            const drawer = document.getElementById('detail-drawer');
            if (drawer && !drawer.classList.contains('hidden')) {
                const c = drawer.querySelector('.drawer-close');
                if (c) c.click();
                return;
            }
            const ops = document.getElementById('ops-panel');
            if (ops && !ops.classList.contains('hidden')) {
                ops.classList.add('hidden');
                return;
            }
            if (leaderActive) { hideLeader(); return; }
        }

        // Anything below is inactive while typing.
        if (isTypingTarget(e.target)) return;
        if (isOpen()) return;
        if (e.metaKey || e.ctrlKey || e.altKey) return;

        // "/" — open palette empty so user sees placeholder.
        if (e.key === '/') {
            e.preventDefault();
            open('');
            return;
        }
        // "?" — open palette with command help.
        if (e.key === '?') {
            e.preventDefault();
            open('>');
            return;
        }

        // Leader-mode chain: "g" then {d,j,c,a,x}.
        if (leaderActive) {
            switch (e.key) {
                case 'd': e.preventDefault(); hideLeader(); go('/');           return;
                case 'c': e.preventDefault(); hideLeader(); go('/companies');  return;
                case 'j': e.preventDefault(); hideLeader(); go('/jobs');       return;
                case 'a': e.preventDefault(); hideLeader(); go('/activity');   return;
                case 'x': e.preventDefault(); hideLeader(); go('/decisions');  return;
                default: hideLeader(); return;
            }
        }
        if (e.key === 'g') {
            e.preventDefault();
            leaderActive = true;
            showLeader();
            clearTimeout(leaderTimer);
            leaderTimer = setTimeout(hideLeader, 1500);
            return;
        }
    }

    // ─── Wire up. ───────────────────────────────────────────────────────────
    function init() {
        const inp = $input();
        const res = $results();
        const back = $backdrop();
        if (inp) {
            inp.addEventListener('input', onInput);
            inp.addEventListener('keydown', onKeydownInPalette);
        }
        if (res) res.addEventListener('click', onResultsClick);
        if (back) back.addEventListener('click', onBackdropClick);
        document.addEventListener('keydown', onGlobalKeydown);
        // Prefetch index in the background so first open is instant.
        ensureIndex();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
