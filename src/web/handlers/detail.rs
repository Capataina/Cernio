//! Side-drawer detail handlers — return HTML fragments (no chrome) for one
//! job or one company. Mounted at GET /detail/job/:id and /detail/company/:id.
//!
//! The drawer shell lives in `templates::page_with`; static/js/drawer.js fetches
//! these fragments and swaps them into `.drawer-body`.

use axum::extract::{Path, State};
use axum::response::Html;
use maud::{html, Markup, PreEscaped};
use rusqlite::Connection;
use std::sync::Arc;

use crate::data::lane::all_lanes;
use crate::data::models::{CompanyRow, JobRow};
use crate::web::templates::{grade_pill, lane_chip};
use crate::web::AppState;

// ── Public route handlers ─────────────────────────────────────────────────

pub async fn job_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Html<String> {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return Html(error_body("Could not open database.").into_string());
    };
    let Some(job) = fetch_job_by_id(&conn, id) else {
        return Html(error_body("Job not found.").into_string());
    };
    Html(render_job(&job).into_string())
}

pub async fn company_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Html<String> {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return Html(error_body("Could not open database.").into_string());
    };
    let Some(company) = fetch_company_by_id(&conn, id) else {
        return Html(error_body("Company not found.").into_string());
    };
    let jobs = fetch_jobs_for_company(&conn, id);
    Html(render_company(&company, &jobs).into_string())
}

// ── Decision buttons (shared with jobs.rs render path; duplicated here so
//    the drawer body is self-contained and the table-row handler stays
//    private to jobs.rs) ──────────────────────────────────────────────────

pub fn decision_buttons(id: i64, current: Option<&str>, url: &str) -> Markup {
    let active = |kind: &str| {
        if current == Some(kind) { "btn btn-active" } else { "btn" }
    };
    html! {
        div.decision-buttons {
            a.btn.btn-apply href=(url) target="_blank"
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"applied"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Apply"
            }
            button.(active("watching"))
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"watching"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Watch"
            }
            button.(active("rejected"))
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"rejected"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Reject"
            }
            @if let Some(d) = current {
                span.decision-current { (d) }
            }
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────

fn error_body(msg: &str) -> Markup {
    html! { div.drawer-error { (msg) } }
}

fn render_job(j: &JobRow) -> Markup {
    let lanes = all_lanes(j.lanes.as_deref());
    let posted_rel = j
        .posted_date
        .as_deref()
        .map(relative_date_label)
        .unwrap_or_else(|| "—".to_string());

    html! {
        div.drawer-section.drawer-head-section {
            div.drawer-chips {
                @for lk in &lanes {
                    (lane_chip(lk))
                }
                (grade_pill(j.grade.as_deref()))
            }
            h2.drawer-title { (j.title) }
            div.drawer-meta {
                a.drawer-meta-link href=(format!("/companies?co={}", j.company_id)) { (j.company_name) }
                span.drawer-meta-dot { "·" }
                span { (j.location.as_deref().unwrap_or("—")) }
                @if let Some(rp) = j.remote_policy.as_deref() {
                    @if !rp.is_empty() {
                        span.drawer-meta-dot { "·" }
                        span { (rp) }
                    }
                }
                span.drawer-meta-dot { "·" }
                span title=(j.posted_date.as_deref().unwrap_or("unknown")) { (posted_rel) }
            }
            div.drawer-actions {
                (decision_buttons(j.id, j.decision.as_deref(), &j.url))
            }
        }

        @if let Some(fa) = j.fit_assessment.as_deref() {
            @if !fa.trim().is_empty() {
                div.drawer-section {
                    h3.drawer-section-title { "Fit assessment" }
                    div.drawer-prose { (PreEscaped(linebreaks_to_html(fa))) }
                }
            }
        }

        @if let Some(rd) = j.raw_description.as_deref() {
            @if !rd.trim().is_empty() {
                div.drawer-section {
                    h3.drawer-section-title { "Job description" }
                    div.drawer-description { (PreEscaped(safe_description(rd))) }
                }
            }
        }

        div.drawer-section.drawer-tags {
            @if let Some(eb) = j.evidence_basis.as_deref() {
                @if !eb.is_empty() {
                    span.drawer-tag { "evidence: " (eb) }
                }
            }
            span.drawer-tag { "status: " (j.evaluation_status) }
        }

        @if !j.url.is_empty() {
            div.drawer-section {
                h3.drawer-section-title { "Posting" }
                a.drawer-url href=(j.url) target="_blank" rel="noopener" { (j.url) }
            }
        }
    }
}

fn render_company(c: &CompanyRow, jobs: &[JobRow]) -> Markup {
    let lanes = all_lanes(c.lanes.as_deref());
    let sponsor = c.sponsors_uk.as_deref().unwrap_or("unknown");
    let sponsor_class = match sponsor {
        "yes" => "drawer-tag drawer-tag-good",
        "no" => "drawer-tag drawer-tag-bad",
        _ => "drawer-tag",
    };
    let status_class = match c.status.as_str() {
        "resolved" => "status-pill status-resolved",
        "bespoke" => "status-pill status-bespoke",
        "potential" => "status-pill status-potential",
        "archived" => "status-pill status-archived",
        _ => "status-pill",
    };

    html! {
        div.drawer-section.drawer-head-section {
            div.drawer-chips {
                @for lk in &lanes {
                    (lane_chip(lk))
                }
                (grade_pill(c.grade.as_deref()))
                span.(status_class) { (c.status) }
            }
            h2.drawer-title {
                a.drawer-title-link href=(c.website) target="_blank" rel="noopener" { (c.name) }
            }
            div.drawer-meta {
                span { (c.location.as_deref().unwrap_or("—")) }
                span.drawer-meta-dot { "·" }
                span class=(sponsor_class) { "sponsors UK: " (sponsor) }
            }
        }

        @if !c.what_they_do.trim().is_empty() {
            div.drawer-section {
                h3.drawer-section-title { "What they do" }
                div.drawer-prose { (PreEscaped(linebreaks_to_html(&c.what_they_do))) }
            }
        }

        @if let Some(gr) = c.grade_reasoning.as_deref() {
            @if !gr.trim().is_empty() {
                div.drawer-section {
                    h3.drawer-section-title { "Grade reasoning" }
                    div.drawer-prose { (PreEscaped(linebreaks_to_html(gr))) }
                }
            }
        }

        @if !c.why_relevant.trim().is_empty() {
            div.drawer-section {
                h3.drawer-section-title { "Why relevant" }
                div.drawer-prose { (PreEscaped(linebreaks_to_html(&c.why_relevant))) }
            }
        }

        div.drawer-section {
            h3.drawer-section-title { "ATS" }
            div.drawer-kv {
                span.drawer-kv-key { "provider" }
                span.drawer-kv-val { (c.ats_provider.as_deref().unwrap_or("—")) }
                span.drawer-kv-key { "slug" }
                span.drawer-kv-val { (c.ats_slug.as_deref().unwrap_or("—")) }
                @if let Some(curl) = c.careers_url.as_deref() {
                    @if !curl.is_empty() {
                        span.drawer-kv-key { "careers" }
                        span.drawer-kv-val {
                            a href=(curl) target="_blank" rel="noopener" { (curl) }
                        }
                    }
                }
            }
        }

        div.drawer-section {
            h3.drawer-section-title { "Jobs at this company · " (jobs.len()) }
            @if jobs.is_empty() {
                div.drawer-empty { "No active jobs." }
            } @else {
                ul.drawer-job-list {
                    @for j in jobs {
                        li.drawer-job-item data-detail=(format!("job-{}", j.id)) {
                            (grade_pill(j.grade.as_deref()))
                            span.drawer-job-title { (j.title) }
                            span.drawer-job-loc { (j.location.as_deref().unwrap_or("—")) }
                        }
                    }
                }
            }
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn linebreaks_to_html(s: &str) -> String {
    // Escape, then turn \n into <br>. Trim trailing whitespace on each line.
    let escaped = html_escape(s);
    escaped.replace('\n', "<br>\n")
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Raw descriptions come from controlled scrapes; pass through as HTML so
/// `<p>`, `<ul>`, `<li>`, `<br>` render. If the content has no HTML tags,
/// convert linebreaks. Crude detection: any `<` followed by an alpha char.
fn safe_description(s: &str) -> String {
    let looks_like_html = s.contains("<p") || s.contains("<ul") || s.contains("<li")
        || s.contains("<br") || s.contains("<div") || s.contains("<h");
    if looks_like_html {
        s.to_string()
    } else {
        linebreaks_to_html(s)
    }
}

fn relative_date_label(s: &str) -> String {
    let now = chrono::Utc::now().naive_utc();
    let parsed = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap_or_default())
        .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok())
        .or_else(|| chrono::DateTime::parse_from_rfc3339(s).ok().map(|d| d.naive_utc()));
    let Some(dt) = parsed else { return s.to_string() };
    let days = (now - dt).num_days();
    if days < 0 { "today".to_string() }
    else if days == 0 { "today".to_string() }
    else if days == 1 { "1d ago".to_string() }
    else if days < 30 { format!("{days}d ago") }
    else if days < 365 { format!("{}mo ago", days / 30) }
    else { format!("{}y ago", days / 365) }
}

// ── DB access ──────────────────────────────────────────────────────────────

fn fetch_job_by_id(conn: &Connection, id: i64) -> Option<JobRow> {
    let sql = "SELECT j.id, j.company_id, c.name, j.title, j.url, j.location, j.remote_policy,
                      j.posted_date, j.evaluation_status, j.fit_assessment, j.grade,
                      j.evidence_basis, j.raw_description,
                      (SELECT decision FROM user_decisions WHERE job_id = j.id
                       ORDER BY decided_at DESC LIMIT 1),
                      0 AS has_package,
                      j.lanes
               FROM jobs j JOIN companies c ON c.id = j.company_id
               WHERE j.id = ?1";
    conn.query_row(sql, [id], |row| {
        Ok(JobRow {
            id: row.get(0)?,
            company_id: row.get(1)?,
            company_name: row.get(2)?,
            title: row.get(3)?,
            url: row.get(4)?,
            location: row.get(5)?,
            remote_policy: row.get(6)?,
            posted_date: row.get(7)?,
            evaluation_status: row.get(8)?,
            fit_assessment: row.get(9)?,
            grade: row.get(10)?,
            evidence_basis: row.get(11)?,
            raw_description: row.get(12)?,
            decision: row.get(13)?,
            has_package: false,
            lanes: row.get(15)?,
        })
    })
    .ok()
}

fn fetch_company_by_id(conn: &Connection, id: i64) -> Option<CompanyRow> {
    let sql = "SELECT c.id, c.name, c.website, c.what_they_do, c.status, c.location,
                      c.sector_tags, c.grade, c.grade_reasoning, c.why_relevant, c.careers_url,
                      (SELECT p.ats_provider FROM company_portals p
                       WHERE p.company_id = c.id AND p.is_primary = 1 LIMIT 1),
                      (SELECT p.ats_slug FROM company_portals p
                       WHERE p.company_id = c.id AND p.is_primary = 1 LIMIT 1),
                      (SELECT COUNT(*) FROM jobs j WHERE j.company_id = c.id),
                      (SELECT COUNT(*) FROM jobs j WHERE j.company_id = c.id
                       AND j.grade IN ('SS', 'S', 'A')),
                      c.lanes, c.sponsors_uk
               FROM companies c WHERE c.id = ?1";
    conn.query_row(sql, [id], |row| {
        Ok(CompanyRow {
            id: row.get(0)?,
            name: row.get(1)?,
            website: row.get(2)?,
            what_they_do: row.get(3)?,
            status: row.get(4)?,
            location: row.get(5)?,
            sector_tags: row.get(6)?,
            grade: row.get(7)?,
            grade_reasoning: row.get(8)?,
            why_relevant: row.get(9)?,
            careers_url: row.get(10)?,
            ats_provider: row.get(11)?,
            ats_slug: row.get(12)?,
            job_count: row.get(13)?,
            fit_count: row.get(14)?,
            lanes: row.get(15)?,
            sponsors_uk: row.get(16)?,
        })
    })
    .ok()
}

fn fetch_jobs_for_company(conn: &Connection, company_id: i64) -> Vec<JobRow> {
    let sql = "SELECT j.id, j.company_id, c.name, j.title, j.url, j.location, j.remote_policy,
                      j.posted_date, j.evaluation_status, j.fit_assessment, j.grade,
                      j.evidence_basis, NULL AS raw_description,
                      (SELECT decision FROM user_decisions WHERE job_id = j.id
                       ORDER BY decided_at DESC LIMIT 1),
                      j.lanes
               FROM jobs j JOIN companies c ON c.id = j.company_id
               WHERE j.company_id = ?1 AND j.evaluation_status != 'archived'
               ORDER BY CASE j.grade
                   WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                   WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6 ELSE 7
               END, j.title";
    let mut stmt = match conn.prepare(sql) { Ok(s) => s, Err(_) => return Vec::new() };
    stmt.query_map([company_id], |row| {
        Ok(JobRow {
            id: row.get(0)?,
            company_id: row.get(1)?,
            company_name: row.get(2)?,
            title: row.get(3)?,
            url: row.get(4)?,
            location: row.get(5)?,
            remote_policy: row.get(6)?,
            posted_date: row.get(7)?,
            evaluation_status: row.get(8)?,
            fit_assessment: row.get(9)?,
            grade: row.get(10)?,
            evidence_basis: row.get(11)?,
            raw_description: row.get(12)?,
            decision: row.get(13)?,
            has_package: false,
            lanes: row.get(14)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}
