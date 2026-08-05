mod app;
mod config;
mod file_explorer;
mod git_utils;
mod ui;

use app::{App, AppMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io::{self, BufWriter}};
use tui_input::backend::crossterm::EventHandler;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let mut stdout = BufWriter::with_capacity(1024 * 1024, stdout);
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    let stdout = io::stdout();
    let mut stdout = BufWriter::with_capacity(1024 * 1024, stdout);
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<BufWriter<std::io::Stdout>>>,
    app: &mut App,
) -> io::Result<()> {
    let mut needs_redraw = true;
    loop {
        if app.check_background_tasks() {
            needs_redraw = true;
        }
        
        if needs_redraw || app.is_loading {
            terminal.draw(|f| ui::ui(f, app))?;
            needs_redraw = false;
        }

        if let Some(t) = app.last_search_typing {
            if t.elapsed() >= std::time::Duration::from_millis(300) {
                app.apply_search_debounce();
                app.last_search_typing = None;
                needs_redraw = true;
            }
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            loop {
                match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    needs_redraw = true;
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        app.should_quit = true;
                        continue;
                    }
                    if app.flash_message.is_some() {
                        app.flash_message = None;
                        continue;
                    }
                    match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Right if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
                        app.enter_commits_view();
                    },
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Char('j') | KeyCode::Down => app.next_item(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_item(),
                    KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter_details(),
                    KeyCode::Char('a') => app.enter_input_mode(AppMode::InputAuthor),
                    KeyCode::Char('d') => app.enter_input_mode(AppMode::InputDate),
                    KeyCode::Char('p') => app.enter_input_mode(AppMode::FileExplorer),
                    KeyCode::Char('P') => app.enter_input_mode(AppMode::InputProfile),
                    KeyCode::Char(' ') => app.toggle_project(),
                    KeyCode::Char('s') => app.toggle_all_projects(),
                    KeyCode::Char('c') => app.toggle_expand(),
                    KeyCode::Char('b') => app.enter_input_mode(AppMode::InputBranch),
                    KeyCode::Char('R') => app.reload_data(),
                    KeyCode::Char('r') | KeyCode::Delete => app.remove_project(),
                    KeyCode::Char('D') => app.remove_all_projects(),
                    KeyCode::Char('u') => app.mode = AppMode::ConfirmPush { force: false },
                    KeyCode::Char('U') => app.mode = AppMode::ConfirmPush { force: true },
                    KeyCode::Char('A') => app.enter_input_mode(AppMode::InputAddSource),
                    KeyCode::Char('.') => {
                        if let Ok(cwd) = std::env::current_dir() {
                            app.add_source(cwd);
                        }
                    }
                    KeyCode::Char('e') => {
                        app.trigger_export();
                    }
                    KeyCode::Char('v') => {
                        app.mode = AppMode::Dashboard;
                    }
                    KeyCode::Char(':') => app.enter_input_mode(AppMode::Command),
                    KeyCode::Char('G') => {
                        let last = app.projects.len().saturating_sub(1);
                        app.project_list_state.select(Some(last));
                        app.selected_project_idx = Some(last);
                    }
                    KeyCode::Char('g') => {
                        if !app.projects.is_empty() {
                            app.project_list_state.select(Some(0));
                            app.selected_project_idx = Some(0);
                        }
                    }
                    KeyCode::Char('H') => {
                        app.hide_zero_commits = !app.hide_zero_commits;
                    }
                    KeyCode::Char('?') => app.mode = AppMode::Help,
                    KeyCode::Char('/') => {
                        app.enter_input_mode(AppMode::Search(crate::app::SearchTarget::Projects));
                    }
                    _ => {}
                },
                AppMode::CommitsView | AppMode::Details => {
                    match key.code {
                        KeyCode::Left if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
                            app.leave_details();
                        },
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                            if app.diff_content.is_some() {
                                app.diff_content = None;
                            } else {
                                app.leave_details();
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.next_item();
                            if app.diff_content.is_some() {
                                app.show_diff_for_selected();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.previous_item();
                            if app.diff_content.is_some() {
                                app.show_diff_for_selected();
                            }
                        }
                        KeyCode::Enter | KeyCode::Char('l') => {
                            app.show_diff_for_selected();
                        }
                        KeyCode::PageDown => {
                            app.diff_scroll = app.diff_scroll.saturating_add(5);
                        }
                        KeyCode::PageUp => {
                            app.diff_scroll = app.diff_scroll.saturating_sub(5);
                        }
                        KeyCode::Char('e') => {
                            app.trigger_export();
                        }
                        KeyCode::Char('c') => app.toggle_expand_from_commits_view(),
                        KeyCode::Char('/') => {
                            app.enter_input_mode(AppMode::Search(crate::app::SearchTarget::Commits));
                        }
                        KeyCode::Char('E') => {
                            if let Some(idx) = app.commit_list_state.selected() {
                                if idx < app.commit_list_map.len() {
                                    let (date_idx, proj_idx, _, _, _) = app.commit_list_map[idx];
                                    if let Some(p_idx) = proj_idx {
                                        if date_idx < app.timeline.len() && p_idx < app.timeline[date_idx].projects.len() {
                                            let proj_name = &app.timeline[date_idx].projects[p_idx].name;
                                            if let Some(proj) = app.projects.iter().find(|p| p.name == *proj_name) {
                                                let proj_path = proj.path.clone();
                                        
                                        // Suspend UI
                                        let _ = crossterm::terminal::disable_raw_mode();
                                        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
                                        
                                        // Run interactive command
                                        let _ = std::process::Command::new("git")
                                            .arg("commit")
                                            .arg("--amend")
                                            .current_dir(&proj_path)
                                            .status();
                                            
                                        // Restore UI
                                        let _ = crossterm::terminal::enable_raw_mode();
                                        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture);
                                        let _ = terminal.clear();
                                        
                                        app.reload_data();
                                        app.flash_message = Some("Amended commit!".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char(':') => app.enter_input_mode(AppMode::Command),
                        KeyCode::Char('G') => {
                            let last = app.commit_list_map.len().saturating_sub(1);
                            app.commit_list_state.select(Some(last));
                        }
                        KeyCode::Char('g') => {
                            if !app.commit_list_map.is_empty() {
                                app.commit_list_state.select(Some(0));
                            }
                        }
                        KeyCode::Char('?') => app.mode = AppMode::Help,
                        _ => {}
                    }
                },
                AppMode::InputAuthor => {
                    match key.code {
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Down => app.next_item(),
                        KeyCode::Up => app.previous_item(),
                        KeyCode::Char('j') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.next_item(),
                        KeyCode::Char('k') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.previous_item(),
                        _ => {
                            app.input.handle_event(&Event::Key(key));
                            app.author_list_state.select(Some(0));
                        }
                    }
                },
                AppMode::ConfirmQuit => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => app.should_quit = true,
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.mode = AppMode::Normal,
                    _ => {}
                },
                AppMode::Dashboard => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => app.mode = AppMode::Normal,
                    KeyCode::Char('j') | KeyCode::Down => {
                        let i = match app.dashboard_list_state.selected() {
                            Some(i) => i + 1,
                            None => 0,
                        };
                        app.dashboard_list_state.select(Some(i));
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let i = match app.dashboard_list_state.selected() {
                            Some(i) => i.saturating_sub(1),
                            None => 0,
                        };
                        app.dashboard_list_state.select(Some(i));
                    }
                    KeyCode::Char('t') => {
                        app.dashboard_resolution = match app.dashboard_resolution {
                            crate::app::TimeResolution::Day => crate::app::TimeResolution::Week,
                            crate::app::TimeResolution::Week => crate::app::TimeResolution::Month,
                            crate::app::TimeResolution::Month => crate::app::TimeResolution::Day,
                        };
                    }
                    KeyCode::Char(':') => app.enter_input_mode(AppMode::Command),
                    KeyCode::Char('/') => app.enter_input_mode(AppMode::Search(crate::app::SearchTarget::Projects)),
                    _ => {}
                },
                AppMode::Help => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => app.mode = AppMode::Normal,
                        _ => {}
                    }
                },
                AppMode::InputDate | AppMode::InputProfile | AppMode::InputAddSource | AppMode::InputExportPath | AppMode::ExplorerJumpPath | AppMode::InputBranch | AppMode::Command => {
                    match key.code {
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Esc => app.cancel_input(),
                        _ => {
                            app.input.handle_event(&Event::Key(key));
                        }
                    }
                },
                AppMode::Search(target) => {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            if key.code == KeyCode::Esc {
                                if target == crate::app::SearchTarget::Projects {
                                    app.project_search.clear();
                                    app.update_visible_projects();
                                } else {
                                    app.fuzzy_filter.clear();
                                    app.build_timeline();
                                }
                            } else {
                                // apply final
                                app.apply_search_debounce();
                            }
                            app.cancel_input();
                        }
                        KeyCode::Down => app.next_item(),
                        KeyCode::Up => app.previous_item(),
                        KeyCode::Char('j') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.next_item(),
                        KeyCode::Char('k') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.previous_item(),
                        _ => {
                            let old = app.input.value().to_string();
                            app.input.handle_event(&Event::Key(key));
                            if old != app.input.value() {
                                app.last_search_typing = Some(std::time::Instant::now());
                            }
                        }
                    }
                },
                AppMode::FileExplorer => {
                    if app.file_explorer.is_searching {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.file_explorer.is_searching = false;
                            },
                            _ => {
                                app.file_explorer.search_input.handle_event(&Event::Key(key));
                                app.file_explorer.apply_filter();
                            }
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.cancel_input(),
                            KeyCode::Char('j') | KeyCode::Down => { app.file_explorer.vim_buffer.clear(); app.file_explorer.next(); }
                            KeyCode::Char('k') | KeyCode::Up => { app.file_explorer.vim_buffer.clear(); app.file_explorer.previous(); }
                            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => { app.file_explorer.vim_buffer.clear(); app.file_explorer.enter_directory(); }
                            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => { app.file_explorer.vim_buffer.clear(); app.file_explorer.go_up(); }
                            KeyCode::Char('s') => {
                                if let Some(path) = app.file_explorer.get_selected_path() {
                                    app.add_source(path);
                                    app.flash_message = Some("Source added!".to_string());
                                    app.cancel_input(); // back to normal mode
                                }
                            },
                            KeyCode::Char('S') => {
                                let path = app.file_explorer.current_path.clone();
                                app.add_source(path);
                                app.flash_message = Some("Current explorer directory added!".to_string());
                                app.cancel_input();
                            },
                            KeyCode::Char(':') => {
                                app.enter_input_mode(AppMode::ExplorerJumpPath);
                            },
                            KeyCode::Char('/') => {
                                app.file_explorer.is_searching = true;
                                app.file_explorer.search_input.reset();
                            },
                            KeyCode::Char(c) if app.file_explorer.vim_buffer == "m" || app.file_explorer.vim_buffer == "'" => {
                                app.file_explorer.handle_vim_key(c);
                            },
                            KeyCode::Char(c) if c.is_numeric() || matches!(c, 'm' | '\'' | 'g' | 'G') => {
                                app.file_explorer.handle_vim_key(c);
                            },
                            _ => { app.file_explorer.vim_buffer.clear(); }
                        }
                    }
                }
                AppMode::ConfirmPush { force } => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            app.push_project(force);
                            app.mode = AppMode::Normal;
                            continue; // skip clearing flash_message below
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            app.mode = AppMode::Normal;
                        }
                        _ => {}
                    }
                }
            }
            
            match app.mode {
                AppMode::Normal | AppMode::Details => {
                    app.flash_message = None;
                }
                _ => {}
            }
        }
                Event::Mouse(mouse_event) => {
                    app.handle_mouse_event(mouse_event);
                    needs_redraw = true;
                }
                _ => {}
                }
                if app.should_quit || !event::poll(std::time::Duration::from_millis(0))? {
                    break;
                }
            }
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use std::path::PathBuf;
    use app::LoadingResult;

    #[test]
    fn test_scan_performance() {
        let mut app = app::App::new();
        app.sources = vec![PathBuf::from("/home/mahefa/git")];
        println!("Starting parallel Rayon scan on /home/mahefa/git...");
        let start = Instant::now();
        app.scan_sources();
        
        if let Some(rx) = &app.loading_rx {
            let mut count = 0;
            loop {
                if let Ok(res) = rx.recv() {
                    match res {
                        LoadingResult::ProjectScanned(_) => count += 1,
                        LoadingResult::ScanComplete(_) => break,
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            println!("Scan complete! Parsed {} repositories in {:?}", count, start.elapsed());
        }
    }
}
