use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::tui::app::{App, Focus, PipelineColumn, View};

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
                    0 => PipelineColumn::Watching,
                    1 => PipelineColumn::Applied,
                    _ => PipelineColumn::Interview,
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
