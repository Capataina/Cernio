/* filters-toggle.js — collapsible filter strip
 *
 * Adds an expand/collapse toggle to .filter-strip on /jobs and /companies.
 * State persists in localStorage under `cernio.filters.<page>.expanded`.
 *
 * Default state:
 *   - collapsed when no filters are active in the URL,
 *   - expanded otherwise (so first-time users immediately see what's filtered).
 *
 * Keyboard shortcut: `f` toggles the strip on the current page. Suppressed
 * inside inputs / textareas / contenteditable to avoid stealing typing.
 */

(function () {
    'use strict';

    const STRIP_SEL = '.filter-strip[data-page]';

    function $strip() { return document.querySelector(STRIP_SEL); }
    function page(strip) { return strip.getAttribute('data-page') || 'default'; }
    function storageKey(p) { return 'cernio.filters.' + p + '.expanded'; }

    function urlHasFilters(p) {
        // Cheap heuristic: any query param other than `detail` (drawer) signals
        // an active filter; archive=active is the default and stripped server-side.
        const params = new URLSearchParams(window.location.search);
        for (const k of params.keys()) {
            if (k === 'detail') continue;
            return true;
        }
        return false;
    }

    function readStored(p) {
        try {
            const v = localStorage.getItem(storageKey(p));
            if (v === null) return null;
            return v === 'true';
        } catch (_) { return null; }
    }

    function writeStored(p, expanded) {
        try {
            localStorage.setItem(storageKey(p), expanded ? 'true' : 'false');
        } catch (_) { /* private mode etc. — silently no-op */ }
    }

    function apply(strip, expanded) {
        strip.classList.toggle('filter-strip-collapsed', !expanded);
        strip.classList.toggle('filter-strip-expanded', expanded);
        const btn = strip.querySelector('.filter-strip-toggle');
        if (btn) {
            btn.setAttribute('aria-expanded', expanded ? 'true' : 'false');
            const chev = btn.querySelector('.filter-strip-chevron');
            if (chev) chev.textContent = expanded ? '▲' : '▼';
        }
    }

    function init() {
        const strip = $strip();
        if (!strip) return;
        const p = page(strip);
        const stored = readStored(p);
        const initial = stored !== null ? stored : urlHasFilters(p);
        apply(strip, initial);

        const btn = strip.querySelector('.filter-strip-toggle');
        if (btn) {
            btn.addEventListener('click', function (e) {
                e.preventDefault();
                const expanded = !strip.classList.contains('filter-strip-expanded');
                apply(strip, expanded);
                writeStored(p, expanded);
            });
        }
    }

    function isTypingTarget(el) {
        if (!el) return false;
        const tag = el.tagName;
        if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
        if (el.isContentEditable) return true;
        return false;
    }

    function onKey(e) {
        if (e.key !== 'f' && e.key !== 'F') return;
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        if (isTypingTarget(e.target)) return;
        const strip = $strip();
        if (!strip) return;
        e.preventDefault();
        const expanded = !strip.classList.contains('filter-strip-expanded');
        apply(strip, expanded);
        writeStored(page(strip), expanded);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
    document.addEventListener('keydown', onKey);
})();
