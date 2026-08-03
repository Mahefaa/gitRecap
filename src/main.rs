mod app;
mod config;
mod file_explorer;
mod git_utils;
mod ui;

use app::{App, AppMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io};
use tui_input::backend::crossterm::EventHandler;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
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
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        app.check_background_tasks();
        terminal.draw(|f| ui::ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Char('j') | KeyCode::Down => app.next_item(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_item(),
                    KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter_details(),
                    KeyCode::Char('a') => app.enter_input_mode(AppMode::InputAuthor),
                    KeyCode::Char('d') => app.enter_input_mode(AppMode::InputDate),
                    KeyCode::Char('p') => app.enter_input_mode(AppMode::FileExplorer),
                    KeyCode::Char('P') => app.enter_input_mode(AppMode::InputProfile),
                    KeyCode::Char(' ') => app.toggle_project(),
                    KeyCode::Char('r') | KeyCode::Delete => app.remove_project(),
                    KeyCode::Char('u') => app.mode = AppMode::ConfirmPush { force: false },
                    KeyCode::Char('U') => app.mode = AppMode::ConfirmPush { force: true },
                    KeyCode::Char('A') => app.enter_input_mode(AppMode::InputAddSource),
                    KeyCode::Char('.') => {
                        if let Ok(cwd) = std::env::current_dir() {
                            app.add_source(cwd);
                        }
                    }
                    KeyCode::Char('e') => {
                        let _ = app.export_summary("summary.txt");
                    }
                    _ => {}
                },
                AppMode::Details => match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.leave_details(),
                    KeyCode::Char('j') | KeyCode::Down => app.next_item(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_item(),
                    KeyCode::Char('e') => {
                        let _ = app.export_summary("summary.txt");
                    }
                    _ => {}
                },
                AppMode::InputAuthor => {
                    match key.code {
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                        _ => {
                            app.input.handle_event(&Event::Key(key));
                            app.author_list_state.select(Some(0));
                        }
                    }
                },
                AppMode::InputDate | AppMode::InputProfile | AppMode::InputAddSource | AppMode::ExplorerJumpPath => {
                    match key.code {
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Esc => app.cancel_input(),
                        _ => {
                            app.input.handle_event(&Event::Key(key));
                        }
                    }
                },
                AppMode::FileExplorer => {
                    if app.file_explorer.is_searching {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.file_explorer.is_searching = false;
                                app.file_explorer.load_directory();
                            },
                            _ => {
                                app.file_explorer.search_input.handle_event(&Event::Key(key));
                                app.file_explorer.load_directory();
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
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
