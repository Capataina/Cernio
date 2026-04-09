use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::app::{App, Focus, View};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Help overlay swallows all keys — any press dismisses it.
    if app.show_help {
        app.show_help = false;
        return;
    }

    // Grade picker overlay — single-key selection.
    if app.show_grade_picker {
        handle_grade_picker(app, key);
        return;
    }

    // Bulk action picker — pick a grade to apply bulk action.
    if app.show_bulk_picker {
        handle_bulk_picker(app, key);
        return;
    }

    // Search mode — route all input to the search handler.
    if app.search_mode {
        handle_search_input(app, key);
        return;
    }

    // ── Global keys (work in every view) ─────────────────────────

    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
            return;
        }
        KeyCode::Char('?') => {
            app.show_help = true;
            return;
        }
        KeyCode::Char('1') => {
            app.view = View::Dashboard;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('2') => {
            app.view = View::Companies;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('3') => {
            app.view = View::Jobs;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('4') => {
            app.view = View::Pipeline;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('f') => {
            if !matches!(app.view, View::Dashboard | View::Pipeline) {
                app.focused_mode = !app.focused_mode;
                app.hide_applied = app.focused_mode;
                app.add_toast(if app.focused_mode {
                    "Focused: hiding F/C + applied".to_string()
                } else {
                    "Showing all grades + applied".to_string()
                });
                app.refresh();
            }
            return;
        }
        KeyCode::Char('A') => {
            if !matches!(app.view, View::Dashboard) {
                app.show_archived = !app.show_archived;
                app.add_toast(if app.show_archived {
                    "Showing archived".to_string()
                } else {
                    "Hiding archived".to_string()
                });
                app.refresh();
            }
            return;
        }
        KeyCode::Tab => {
            if matches!(app.view, View::Companies | View::Jobs) {
                app.focus = match app.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
                app.detail_scroll = 0;
            }
            return;
        }
        KeyCode::Char('/') => {
            if !matches!(app.view, View::Pipeline) {
                app.search_mode = true;
                app.search_query.clear();
            }
            return;
        }
        KeyCode::Char('y') => {
            app.copy_url_to_clipboard();
            app.add_toast("URL copied".to_string());
            return;
        }
        KeyCode::Char('e') => {
            app.export_current_view();
            return;
        }
        _ => {}
    }

    // ── View + focus–specific keys ───────────────────────────────

    match (app.view, app.focus) {
        (View::Dashboard, _) => handle_dashboard(app, key),
        (View::Companies, Focus::List) => handle_company_list(app, key),
        (View::Companies, Focus::Detail) => handle_detail_scroll(app, key),
        (View::Jobs, Focus::List) => handle_job_list(app, key),
        (View::Jobs, Focus::Detail) => handle_detail_scroll(app, key),
        (View::Pipeline, _) => handle_pipeline(app, key),
    }
}

fn handle_grade_picker(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_grade_picker = false;
        }
        KeyCode::Char(c) => {
            let grade = match c {
                '1' => Some("SS"),
                '2' => Some("S"),
                '3' => Some("A"),
                '4' => Some("B"),
                '5' => Some("C"),
                '6' => Some("F"),
                _ => None,
            };
            if let Some(g) = grade {
                app.override_grade(g);
            } else {
                app.show_grade_picker = false;
            }
        }
        _ => {
            app.show_grade_picker = false;
        }
    }
}

fn handle_bulk_picker(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_bulk_picker = false;
        }
        KeyCode::Char(c) => {
            let grade = match c {
                '1' => Some("SS"),
                '2' => Some("S"),
                '3' => Some("A"),
                '4' => Some("B"),
                '5' => Some("C"),
                '6' => Some("F"),
                _ => None,
            };
            if let Some(g) = grade {
                let action = app.bulk_action.clone();
                app.bulk_decision_by_grade(g, &action);
            }
            app.show_bulk_picker = false;
        }
        _ => {
            app.show_bulk_picker = false;
        }
    }
}

fn handle_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.search_mode = false;
            app.search_query.clear();
            app.refresh();
        }
        KeyCode::Enter => {
            app.search_mode = false;
            // Keep filter active — do not clear search_query.
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.refresh();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.refresh();
        }
        _ => {}
    }
}

fn handle_dashboard(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_viewport_down(1),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_viewport_up(1),
        KeyCode::Char('D') => app.run_cleanup(),
        _ => {}
    }
}

fn handle_company_list(app: &mut App, key: KeyEvent) {
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

fn handle_job_list(app: &mut App, key: KeyEvent) {
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

fn handle_pipeline(app: &mut App, key: KeyEvent) {
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
                super::app::PipelineColumn::Watching => app.pipeline_watching.get(sel).map(|c| c.job_id),
                super::app::PipelineColumn::Applied => app.pipeline_applied.get(sel).map(|c| c.job_id),
                super::app::PipelineColumn::Interview => app.pipeline_interview.get(sel).map(|c| c.job_id),
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

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    if app.show_help || app.show_grade_picker || app.show_bulk_picker {
        return;
    }

    // Determine which pane the mouse is in.
    // If list_area hasn't been set yet (first frame), use terminal midpoint as split.
    let list_right = if app.list_area.width > 0 {
        app.list_area.x + app.list_area.width
    } else {
        app.terminal_width / 2
    };
    let detail_left = if app.detail_area.width > 0 {
        app.detail_area.x
    } else {
        app.terminal_width / 2
    };

    match mouse.kind {
        MouseEventKind::ScrollDown => {
            match app.view {
                View::Dashboard => {
                    app.scroll_viewport_down(3);
                }
                View::Pipeline => {
                    app.pipeline_next();
                    app.pipeline_next();
                    app.pipeline_next();
                }
                _ => {
                    if mouse.column < list_right {
                        app.scroll_table_down(3);
                    } else if mouse.column >= detail_left {
                        app.scroll_detail_down();
                        app.scroll_detail_down();
                        app.scroll_detail_down();
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            match app.view {
                View::Dashboard => {
                    app.scroll_viewport_up(3);
                }
                View::Pipeline => {
                    app.pipeline_prev();
                    app.pipeline_prev();
                    app.pipeline_prev();
                }
                _ => {
                    if mouse.column < list_right {
                        app.scroll_table_up(3);
                    } else if mouse.column >= detail_left {
                        app.scroll_detail_up();
                        app.scroll_detail_up();
                        app.scroll_detail_up();
                    }
                }
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
            let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);

            // Click on tab bar (rows 0–2).
            if mouse.row < 3 {
                let col = mouse.column;
                if col < 15 {
                    app.view = View::Dashboard;
                } else if col < 35 {
                    app.view = View::Companies;
                } else if col < 55 {
                    app.view = View::Jobs;
                } else {
                    app.view = View::Pipeline;
                }
                app.focus = Focus::List;
                app.detail_scroll = 0;
                app.clear_multi_select();
                return;
            }

            // Below tab bar.
            if matches!(app.view, View::Dashboard) {
                return;
            }

            // Pipeline click — select card in the clicked column.
            if matches!(app.view, View::Pipeline) {
                let third = app.terminal_width / 3;
                let col_idx = if mouse.column < third { 0 }
                    else if mouse.column < third * 2 { 1 }
                    else { 2 };
                app.pipeline_column = match col_idx {
                    0 => super::app::PipelineColumn::Watching,
                    1 => super::app::PipelineColumn::Applied,
                    _ => super::app::PipelineColumn::Interview,
                };
                // Rough row calc: each card is ~3 lines (title + company + separator).
                let content_start = 4u16; // tab bar (3) + border (1)
                if mouse.row >= content_start {
                    let visual = (mouse.row - content_start) as usize;
                    let card_idx = visual / 3; // each card ~3 lines
                    let col_len = app.pipeline_col_len();
                    if card_idx < col_len {
                        app.pipeline_selections[col_idx] = card_idx;
                    }
                }
                return;
            }

            let in_left = mouse.column < list_right;

            if in_left {
                app.focus = Focus::List;
                let list_y = if app.list_area.height > 0 { app.list_area.y } else { 3 };
                let table_content_start = list_y + 2;
                if mouse.row >= table_content_start {
                    let visual_row = (mouse.row - table_content_start) as usize;
                    let scroll_offset = match app.view {
                        View::Companies => app.company_state.offset(),
                        View::Jobs => app.job_state.offset(),
                        _ => 0,
                    };
                    let actual_index = visual_row + scroll_offset;
                    let len = match app.view {
                        View::Companies => app.companies.len(),
                        View::Jobs => app.jobs.len(),
                        _ => 0,
                    };
                    if actual_index < len {
                        if ctrl {
                            // Ctrl+click: toggle individual item.
                            app.toggle_multi_select(actual_index);
                        } else if shift {
                            // Shift+click: range select from anchor.
                            app.range_select(actual_index);
                        } else {
                            // Normal click: clear multi-select and select single item.
                            app.clear_multi_select();
                        }

                        // Always update the primary selection.
                        match app.view {
                            View::Companies => {
                                app.company_state.select(Some(actual_index));
                                if !ctrl { app.anchor_company = Some(actual_index); }
                            }
                            View::Jobs => {
                                app.job_state.select(Some(actual_index));
                                if !ctrl { app.anchor_job = Some(actual_index); }
                            }
                            _ => {}
                        }
                        app.detail_scroll = 0;
                    }
                }
            } else {
                app.focus = Focus::Detail;
            }
        }
        _ => {}
    }
}

fn handle_detail_scroll(app: &mut App, key: KeyEvent) {
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
            app.focus = Focus::List;
            app.detail_scroll = 0;
        }
        _ => {}
    }
}
