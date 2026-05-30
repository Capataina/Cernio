/* drawer.js — side-drawer detail view controller.
 *
 * Responsibilities:
 *  - On DOMContentLoaded, if URL has ?detail=kind-id, open the drawer.
 *  - Delegated click handler: any .row-clickable (or descendant of) opens
 *    the drawer using its data-detail token.
 *  - Open: pushState, fetch HTML fragment, swap into .drawer-body, animate.
 *  - Close: Esc / backdrop / .drawer-close → replaceState (strip ?detail),
 *    animate out, hide.
 *  - Skip on Ctrl/Cmd+click and on clicks within action elements (.btn,
 *    a.role-apply, .decision-buttons, links/buttons in general).
 */
(function () {
    'use strict';

    const DRAWER_ID = 'detail-drawer';
    const BACKDROP_ID = 'detail-drawer-backdrop';
    const BODY_SEL = '.drawer-body';
    const KIND_SEL = '.drawer-kind';
    const CLOSE_SEL = '.drawer-close';

    function $drawer() { return document.getElementById(DRAWER_ID); }
    function $backdrop() { return document.getElementById(BACKDROP_ID); }

    function parseToken(token) {
        if (!token) return null;
        // Accepts: "job-123" or "co-456"
        const m = /^(job|co)-(\d+)$/.exec(token);
        if (!m) return null;
        return { kind: m[1], id: Number(m[2]) };
    }

    function fragmentUrl(parsed) {
        if (parsed.kind === 'job') return `/detail/job/${parsed.id}`;
        if (parsed.kind === 'co') return `/detail/company/${parsed.id}`;
        return null;
    }

    function kindLabel(kind) {
        return kind === 'co' ? 'company' : 'job';
    }

    function setUrlParam(token) {
        const url = new URL(window.location.href);
        if (token) {
            url.searchParams.set('detail', token);
        } else {
            url.searchParams.delete('detail');
        }
        return url.pathname + (url.search ? url.search : '') + url.hash;
    }

    let isOpen = false;
    let lastFocus = null;

    async function openDrawer(token, opts) {
        opts = opts || {};
        const parsed = parseToken(token);
        if (!parsed) return;
        const url = fragmentUrl(parsed);
        if (!url) return;

        const drawer = $drawer();
        const backdrop = $backdrop();
        if (!drawer || !backdrop) return;

        // History: push fresh entry on user-initiated open; replace on first-load.
        const newPath = setUrlParam(token);
        if (opts.replace) {
            history.replaceState({ drawer: token }, '', newPath);
        } else {
            history.pushState({ drawer: token }, '', newPath);
        }

        // Show shells immediately so the slide animation runs.
        drawer.classList.remove('hidden');
        backdrop.classList.remove('hidden');
        // Force a reflow so the .visible class triggers the transition.
        // (toggling from display:none directly to a class doesn't always animate.)
        void drawer.offsetWidth;
        drawer.classList.add('visible');
        backdrop.classList.add('visible');

        const kindEl = drawer.querySelector(KIND_SEL);
        if (kindEl) kindEl.textContent = kindLabel(parsed.kind);

        const bodyEl = drawer.querySelector(BODY_SEL);
        if (bodyEl) bodyEl.innerHTML = '<div class="drawer-empty">loading…</div>';

        isOpen = true;
        lastFocus = document.activeElement;

        try {
            const resp = await fetch(url, { headers: { 'Accept': 'text/html' } });
            if (!resp.ok) {
                bodyEl.innerHTML = `<div class="drawer-error">Failed to load (${resp.status}).</div>`;
                return;
            }
            const html = await resp.text();
            bodyEl.innerHTML = html;
            // HTMX needs to process the swapped-in decision buttons so the
            // Apply/Watch/Reject POSTs work.
            if (window.htmx && typeof window.htmx.process === 'function') {
                window.htmx.process(bodyEl);
            }
            // Scroll to top of the drawer body on each open.
            bodyEl.scrollTop = 0;
        } catch (err) {
            if (bodyEl) {
                bodyEl.innerHTML = '<div class="drawer-error">Network error loading detail.</div>';
            }
        }
    }

    function closeDrawer(opts) {
        opts = opts || {};
        const drawer = $drawer();
        const backdrop = $backdrop();
        if (!drawer || !backdrop) return;
        if (!isOpen) return;
        isOpen = false;

        drawer.classList.remove('visible');
        backdrop.classList.remove('visible');

        // Drop ?detail from the URL (replaceState — don't push a no-op entry).
        if (!opts.fromHistory) {
            const newPath = setUrlParam(null);
            history.replaceState({}, '', newPath);
        }

        // After the slide-out completes, hide so clicks pass through.
        setTimeout(() => {
            if (!isOpen) {
                drawer.classList.add('hidden');
                backdrop.classList.add('hidden');
            }
        }, 200);

        if (lastFocus && typeof lastFocus.focus === 'function') {
            try { lastFocus.focus(); } catch (_) {}
        }
    }

    // ── Click delegation ────────────────────────────────────────────────────

    function isInteractiveTarget(el) {
        // Don't trigger drawer if the user clicked an action: buttons, anchors,
        // form elements, htmx-driven controls.
        return !!el.closest('button, a, input, select, textarea, .decision-buttons, [hx-post], [hx-get]');
    }

    function tokenFromRow(row) {
        const explicit = row.getAttribute('data-detail');
        if (explicit) return explicit;
        // Fall back to id="job-NN" / id="co-NN"
        const id = row.id || '';
        if (/^job-\d+$/.test(id) || /^co-\d+$/.test(id)) return id;
        return null;
    }

    document.addEventListener('click', function (ev) {
        // Close-button / backdrop handled below as separate targets.
        const closeBtn = ev.target.closest(`#${DRAWER_ID} ${CLOSE_SEL}`);
        if (closeBtn) {
            ev.preventDefault();
            closeDrawer();
            return;
        }
        if (ev.target === $backdrop()) {
            closeDrawer();
            return;
        }

        // Modifier keys → let default behaviour happen.
        if (ev.ctrlKey || ev.metaKey || ev.shiftKey || ev.button !== 0) return;

        // Open from an in-drawer job list item (nested detail navigation).
        const drawerItem = ev.target.closest(`#${DRAWER_ID} .drawer-job-item[data-detail]`);
        if (drawerItem) {
            ev.preventDefault();
            openDrawer(drawerItem.getAttribute('data-detail'));
            return;
        }

        // Generic row trigger.
        const row = ev.target.closest('.row-clickable, .job-row, .company-row');
        if (!row) return;
        if (isInteractiveTarget(ev.target)) return;
        const token = tokenFromRow(row);
        if (!token) return;
        ev.preventDefault();
        openDrawer(token);
    });

    // ── Esc key ─────────────────────────────────────────────────────────────

    document.addEventListener('keydown', function (ev) {
        if (ev.key === 'Escape' && isOpen) {
            ev.preventDefault();
            closeDrawer();
        }
    });

    // ── Back/forward navigation ─────────────────────────────────────────────

    window.addEventListener('popstate', function () {
        const url = new URL(window.location.href);
        const token = url.searchParams.get('detail');
        if (token) {
            openDrawer(token, { replace: true });
        } else if (isOpen) {
            closeDrawer({ fromHistory: true });
        }
    });

    // ── Initial load ────────────────────────────────────────────────────────

    document.addEventListener('DOMContentLoaded', function () {
        const url = new URL(window.location.href);
        const token = url.searchParams.get('detail');
        if (token) {
            openDrawer(token, { replace: true });
        }
    });
})();
