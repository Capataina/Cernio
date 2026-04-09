use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, View};

pub fn handle_dashboard(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_viewport_down(1),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_viewport_up(1),
        KeyCode::Char('D') => app.run_cleanup(),
        _ => {}
    }
}

pub fn handle_company_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_in_list(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_in_list(),
        KeyCode::Char('g') => app.go_to_top(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Home => app.go_to_top(),
        KeyCode::End => app.go_to_bottom(),
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter_company_jobs(),
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Esc => app.clear_multi_select(),
        _ => {}
    }
}

pub fn handle_job_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_in_list(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_in_list(),
        KeyCode::Char('g') => {
            app.show_grade_picker = true;
        }
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Home => app.go_to_top(),
        KeyCode::End => app.go_to_bottom(),
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('p') => app.autofill_selected_job(),
        KeyCode::Char('w') => app.record_decision_multi("watching"),
        KeyCode::Char('a') => app.record_decision_multi("applied"),
        KeyCode::Char('x') => app.record_decision_multi("rejected"),
        KeyCode::Char('i') => app.record_decision_multi("interview"),
        KeyCode::Char('W') => {
            app.bulk_action = "watching".to_string();
            app.show_bulk_picker = true;
        }
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Char(']') => app.jump_next_grade_section(),
        KeyCode::Char('[') => app.jump_prev_grade_section(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            if app.job_filter_company.is_some() {
                app.clear_job_filter();
                app.view = View::Companies;
                app.detail_scroll = 0;
            } else {
                app.clear_multi_select();
            }
        }
        _ => {}
    }
}

pub fn handle_pipeline(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.pipeline_next(),
        KeyCode::Char('k') | KeyCode::Up => app.pipeline_prev(),
        KeyCode::Char('l') | KeyCode::Right => app.pipeline_col_right(),
        KeyCode::Char('h') | KeyCode::Left => app.pipeline_col_left(),
        KeyCode::Char('w') => app.pipeline_move_card("watching"),
        KeyCode::Char('a') => app.pipeline_move_card("applied"),
        KeyCode::Char('i') => app.pipeline_move_card("interview"),
        KeyCode::Char('o') => {
            // Open URL of selected pipeline card.
            let col_idx = app.pipeline_col_index();
            let sel = app.pipeline_selections[col_idx];
            let job_id = match app.pipeline_column {
                crate::tui::app::PipelineColumn::Watching => app.pipeline_watching.get(sel).map(|c| c.job_id),
                crate::tui::app::PipelineColumn::Applied => app.pipeline_applied.get(sel).map(|c| c.job_id),
                crate::tui::app::PipelineColumn::Interview => app.pipeline_interview.get(sel).map(|c| c.job_id),
            };
            if let Some(id) = job_id {
                // Find the job URL.
                if let Some(job) = app.jobs.iter().find(|j| j.id == id) {
                    let _ = std::process::Command::new("open").arg(&job.url).spawn();
                }
            }
        }
        _ => {}
    }
}

pub fn handle_detail_scroll(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_detail_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_detail_up(),
        KeyCode::Home => { app.detail_scroll = 0; }
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('p') if app.view == View::Jobs => app.autofill_selected_job(),
        KeyCode::Char('w') if app.view == View::Jobs => app.record_decision_multi("watching"),
        KeyCode::Char('a') if app.view == View::Jobs => app.record_decision_multi("applied"),
        KeyCode::Char('x') if app.view == View::Jobs => app.record_decision_multi("rejected"),
        KeyCode::Char('i') if app.view == View::Jobs => app.record_decision_multi("interview"),
        KeyCode::Char('s') if app.view == View::Jobs => app.toggle_sort(),
        KeyCode::Char(']') if app.view == View::Jobs => app.jump_next_grade_section(),
        KeyCode::Char('[') if app.view == View::Jobs => app.jump_prev_grade_section(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            app.focus = crate::tui::app::Focus::List;
            app.detail_scroll = 0;
        }
        _ => {}
    }
}
