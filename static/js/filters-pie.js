/* filters-pie.js — lane pie interactions
 *
 * Plain wedge clicks follow the server-rendered href (toggle behaviour).
 * This script adds:
 *   - shift-click on a wedge: "only this lane" — navigate to URL with just
 *     that lane in the lane= param.
 *   - centre-button click: master toggle. If any lane is currently off,
 *     turn ALL lanes on (clear the lane= param). Otherwise turn ALL lanes
 *     off — navigate with `lane=__none__` which produces an empty match;
 *     in practice this lands the user back to "all lanes on" because the
 *     server treats an empty lane filter as no filter, so we explicitly
 *     clear instead. The centre button state-attribute is updated on init.
 */

(function () {
    'use strict';

    const LANE_KEYS = [
        'big-tech', 'ai-ml', 'hft', 'crypto-mm',
        'bank-strats', 'systems-infra', 'devtools', 'fintech'
    ];

    function $pie() { return document.querySelector('.lane-pie'); }

    function pageBase() {
        // Derive from current path: /jobs or /companies (default jobs).
        const p = window.location.pathname;
        if (p.startsWith('/companies')) return '/companies';
        return '/jobs';
    }

    function currentParams() {
        return new URLSearchParams(window.location.search);
    }

    function activeLanesFromUrl() {
        const params = currentParams();
        const raw = params.get('lane');
        if (!raw) return new Set();
        return new Set(raw.split(',').map(s => s.trim()).filter(Boolean));
    }

    function navigateWithLane(laneCsv) {
        const params = currentParams();
        // drawer + any other axes preserved.
        params.delete('lane');
        if (laneCsv && laneCsv.length) params.set('lane', laneCsv);
        const qs = params.toString();
        window.location.href = pageBase() + (qs ? ('?' + qs) : '');
    }

    function onWedgeClick(e) {
        // Only intercept shift-click; let plain clicks follow the href.
        if (!e.shiftKey) return;
        const a = e.currentTarget;
        const lane = a.getAttribute('data-lane');
        if (!lane) return;
        e.preventDefault();
        navigateWithLane(lane);
    }

    function onCentreClick(e) {
        e.preventDefault();
        const active = activeLanesFromUrl();
        // No filter active in URL = effectively "all on" from server's POV.
        const allOnNow = active.size === 0 || active.size === LANE_KEYS.length;
        if (allOnNow) {
            // Toggle to "all off" — we use a sentinel that produces no rows.
            // Set lane to every key explicitly with one removed wouldn't make
            // sense; the user intent is "clear results". Use sentinel `__none__`
            // (no lane matches) so the table empties — server filter treats
            // unknown keys as no-match within the lane intersection.
            navigateWithLane('__none__');
        } else {
            // Some lanes off — go back to all-on by clearing the filter.
            navigateWithLane('');
        }
    }

    function updateCentreState(btn) {
        const active = activeLanesFromUrl();
        let state;
        if (active.size === 0) state = 'all';                // no filter = all
        else if (active.size === LANE_KEYS.length) state = 'all';
        else if (active.size === 1 && active.has('__none__')) state = 'none';
        else state = 'some';
        btn.setAttribute('data-state', state);
        const dot = btn.querySelector('.lane-pie-centre-glyph');
        if (dot) {
            dot.textContent = state === 'all' ? '○'
                : state === 'none' ? '×'
                : '◐';
        }
    }

    function init() {
        const pie = $pie();
        if (!pie) return;
        const wedges = pie.querySelectorAll('.lane-wedge');
        wedges.forEach(w => w.addEventListener('click', onWedgeClick));
        const centre = pie.querySelector('.lane-pie-centre');
        if (centre) {
            centre.addEventListener('click', onCentreClick);
            updateCentreState(centre);
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
