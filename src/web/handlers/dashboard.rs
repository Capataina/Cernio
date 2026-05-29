use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use std::sync::Arc;

use crate::data::lane::{lane_hex, lane_label, LANE_KEYS};
use crate::data::queries;
use crate::web::templates::{grade_pill, lane_chip, lane_legend, page as chrome};
use crate::web::AppState;

pub async fn page(State(state): State<Arc<AppState>>) -> Html<String> {
    let body = render(&state).await;
    Html(chrome("Dashboard", "dashboard", body).into_string())
}

async fn render(state: &AppState) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let stats = queries::fetch_stats(&conn);

    // Per-lane × per-grade data for the chart.
    let grades = ["SS", "S", "A", "B", "C", "F"];
    let mut lane_grade_counts: Vec<(String, Vec<i64>)> = Vec::new();
    for key in LANE_KEYS.iter() {
        let prefix = format!("[\"{key}\"%");
        let mut row = Vec::with_capacity(6);
        for g in &grades {
            let n: i64 = conn.query_row(
                "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND grade = ?2 AND evaluation_status != 'archived'",
                rusqlite::params![prefix, g], |r| r.get(0)).unwrap_or(0);
            row.push(n);
        }
        lane_grade_counts.push((key.to_string(), row));
    }

    // Lane totals + hit-rate.
    let lane_totals: Vec<(String, i64, i64)> = lane_grade_counts.iter().map(|(k, row)| {
        let total = row.iter().sum::<i64>();
        let strong = row[0] + row[1] + row[2];
        (k.clone(), total, strong)
    }).collect();

    // Companies per lane (count).
    let mut companies_per_lane: Vec<(String, i64)> = Vec::new();
    for key in LANE_KEYS.iter() {
        let prefix = format!("[\"{key}\"%");
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM companies WHERE lanes LIKE ?1 AND status != 'archived'",
            rusqlite::params![prefix], |r| r.get(0)).unwrap_or(0);
        companies_per_lane.push((key.to_string(), n));
    }

    // Top SS/S/A roles for the right column.
    let top_roles = fetch_top_roles(&conn);

    // Serialise data for ECharts as a JSON island.
    let chart_data = serde_json::json!({
        "lanes": LANE_KEYS.iter().map(|k| lane_label(k)).collect::<Vec<_>>(),
        "lane_keys": LANE_KEYS,
        "lane_colors": LANE_KEYS.iter().map(|k| lane_hex(k)).collect::<Vec<_>>(),
        "grades": grades,
        "grade_colors": ["#c39df0", "#7adf9a", "#7ea8ff", "#ffc94a", "#ff5c5c", "#666666"],
        "series": lane_grade_counts.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>(),
        "companies": companies_per_lane.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
    });

    html! {
        div.dashboard {
            (lane_legend())

            section.summary-grid {
                div.stat-tile {
                    div.stat-label { "Companies" }
                    div.stat-value { (stats.total_companies) }
                }
                div.stat-tile {
                    div.stat-label { "Jobs" }
                    div.stat-value { (stats.total_jobs) }
                }
                div.stat-tile.tile-strong {
                    div.stat-label { "Strong (SS + S)" }
                    div.stat-value { (stats.jobs_by_grade.iter().filter(|(g,_)| g == "SS" || g == "S").map(|(_,c)| c).sum::<i64>()) }
                }
                div.stat-tile.tile-applied {
                    div.stat-label { "Applied" }
                    div.stat-value { (stats.applied_count) }
                }
                div.stat-tile {
                    div.stat-label { "Watching" }
                    div.stat-value { (stats.watching_count) }
                }
                div.stat-tile {
                    div.stat-label { "Pending eval" }
                    div.stat-value { (stats.jobs_by_eval.iter().filter(|(e,_)| e == "pending").map(|(_,c)| c).sum::<i64>()) }
                }
            }

            section.panel {
                header.panel-head {
                    h2 { "Jobs × Lane × Grade" }
                    span.panel-sub { "active jobs only · hover for counts" }
                }
                div #lane-chart .chart {}
                script type="application/json" #lane-chart-data { (maud::PreEscaped(chart_data.to_string())) }
            }

            div.dual-panels {
                section.panel {
                    header.panel-head { h2 { "Lane hit-rate" } span.panel-sub { "% of lane's jobs landing SS + S + A" } }
                    div.hit-rate-rows {
                        @for (key, total, strong) in &lane_totals {
                            @let pct = if *total > 0 { (strong * 100 / total) as i32 } else { 0 };
                            div.hit-rate-row {
                                (lane_chip(key))
                                div.hit-rate-bar {
                                    div.hit-rate-fill style=(format!("width: {pct}%; --lane-color: {}", lane_hex(key))) {}
                                }
                                span.hit-rate-pct { (pct) "%" }
                                span.hit-rate-total { (strong) " / " (total) }
                            }
                        }
                    }
                }

                section.panel {
                    header.panel-head { h2 { "Top roles" } span.panel-sub { "SS + S + A · click apply to mark + open" } }
                    div.top-roles {
                        @if top_roles.is_empty() {
                            div.empty { "No graded jobs yet." }
                        }
                        @for (grade, title, company, lanes_json, url, id) in &top_roles {
                            div.role-row {
                                (lane_chip_for_opt(lanes_json.as_deref()))
                                (grade_pill(Some(grade.as_str())))
                                div.role-text {
                                    div.role-title { (title) }
                                    div.role-company { (company) }
                                }
                                a.role-apply href=(url) target="_blank"
                                    hx-post=(format!("/jobs/{id}/decision"))
                                    hx-vals=(r#"{"decision":"applied"}"#)
                                    hx-swap="outerHTML" {
                                    "Apply"
                                }
                            }
                        }
                    }
                }
            }
        }

        script {
            (maud::PreEscaped(r#"
            (function() {
              const el = document.getElementById('lane-chart');
              const data = JSON.parse(document.getElementById('lane-chart-data').textContent);
              if (!el || !window.echarts) return;
              const chart = echarts.init(el, null, { renderer: 'canvas' });
              const series = data.grades.map((g, gi) => ({
                name: g,
                type: 'bar',
                stack: 'total',
                emphasis: { focus: 'series' },
                itemStyle: { color: data.grade_colors[gi] },
                data: data.series.map(row => row[gi]),
              }));
              chart.setOption({
                backgroundColor: 'transparent',
                grid: { left: 90, right: 24, top: 28, bottom: 32, containLabel: false },
                tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' }, backgroundColor: '#0c1014', borderColor: '#1f2630', textStyle: { color: '#e8ecf2' } },
                legend: { textStyle: { color: '#aab3bf' }, top: 4, right: 12, icon: 'rect' },
                xAxis: { type: 'value', axisLabel: { color: '#7a838f' }, splitLine: { lineStyle: { color: '#1a1f27' } } },
                yAxis: { type: 'category', data: data.lanes, axisLabel: { color: '#cbd2db', fontFamily: 'JetBrains Mono, monospace', fontSize: 12 }, axisLine: { lineStyle: { color: '#2a3038' } } },
                series: series,
              });
              window.addEventListener('resize', () => chart.resize());
            })();
            "#))
        }
    }
}

fn lane_chip_for_opt(lanes_json: Option<&str>) -> Markup {
    match crate::data::lane::primary_lane(lanes_json) {
        Some(key) => lane_chip(&key),
        None => html! { span.lane-chip.lane-none { "—" } },
    }
}

fn fetch_top_roles(conn: &Connection) -> Vec<(String, String, String, Option<String>, String, i64)> {
    let sql = "
        SELECT j.grade, j.title, c.name, j.lanes, j.url, j.id
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.grade IN ('SS', 'S', 'A')
        AND j.evaluation_status != 'archived'
        AND c.status != 'archived'
        ORDER BY CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 END,
                 j.title
        LIMIT 30
    ";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}
