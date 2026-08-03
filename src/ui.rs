use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, AppMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Top bar
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer / Input area
            ]
            .as_ref(),
        )
        .split(f.area());

    // Top Bar
    let date_display = format!("{}..{}", app.date_start_filter.format("%Y-%m-%d"), app.date_end_filter.format("%Y-%m-%d"));
    let filter_text = format!(
        "Profile: [{}] | Date: {} | Author: '{}' | Sources: {}",
        app.current_profile.name,
        date_display,
        if app.author_filter.is_empty() { "Any" } else { &app.author_filter },
        app.sources.len()
    );
    let top_bar = Paragraph::new(Line::from(vec![
        Span::styled("GitRecap ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("| Projects w/ Commits: {} | ", app.projects.iter().filter(|p| p.enabled && !p.branches.is_empty()).count())),
        Span::styled(filter_text, Style::default().fg(Color::Cyan)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Summary"));
    f.render_widget(top_bar, chunks[0]);

    // Main Content
    match app.mode {
        AppMode::FileExplorer => {
            render_file_explorer(f, app, chunks[1]);
            
            if app.file_explorer.is_searching {
                let input_widget = Paragraph::new(app.file_explorer.search_input.value())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Search (Enter to confirm, Esc to cancel)"));
                f.render_widget(input_widget, chunks[2]);
                #[allow(clippy::cast_possible_truncation)]
                f.set_cursor_position((
                    chunks[2].x + 1 + app.file_explorer.search_input.visual_cursor() as u16,
                    chunks[2].y + 1,
                ));
            } else {
                let footer = Paragraph::new("Esc: Cancel | j/k: Nav | gg/G: Top/Bot | m<c> / '<c>: Marks | s: Add Selected | S: Add Current Dir | : Jump Path | /: Search")
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(footer, chunks[2]);
            }
        },
        AppMode::InputAuthor => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_author_autocomplete(f, app, main_chunks[1]);
            
            let input_widget = Paragraph::new(app.input.value())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Enter Author (Esc to cancel, Enter to submit, j/k to select)"));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 1 + app.input.visual_cursor() as u16,
                chunks[2].y + 1,
            ));
        },
        AppMode::InputDate => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let input_widget = Paragraph::new(app.input.value())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Enter Date [YYYY-MM-DD] or Range [YYYY-MM-DD..YYYY-MM-DD] (Esc cancel, Enter submit)"));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 1 + app.input.visual_cursor() as u16,
                chunks[2].y + 1,
            ));
        },
        AppMode::InputProfile => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let input_widget = Paragraph::new(app.input.value())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Switch or Create Profile (Esc to cancel, Enter to submit)"));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 1 + app.input.visual_cursor() as u16,
                chunks[2].y + 1,
            ));
        },
        AppMode::InputAddSource | AppMode::ExplorerJumpPath => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let title = if matches!(app.mode, AppMode::InputAddSource) {
                "Manually add source path (Esc to cancel, Enter to submit)"
            } else {
                "Jump explorer to path (Esc to cancel, Enter to submit)"
            };
            
            let input_widget = Paragraph::new(app.input.value())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(title));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 1 + app.input.visual_cursor() as u16,
                chunks[2].y + 1,
            ));
        },
        AppMode::Normal | AppMode::Details => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let help_text = match app.mode {
                AppMode::Normal => "q: Quit | P: Profile | a: Author | d: Date | p: Explorer | A: Add Path | l/Enter: Commits | Space: Toggle | c: Collapse | r: Rm | R: Refresh | e: Export | u: Push",
                AppMode::Details => "q: Quit | h/Esc: Back | j/k: Navigate Commits | e: Export",
                _ => "",
            };
            let footer = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        }
        AppMode::ConfirmPush { force } => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let area = centered_rect(50, 20, f.area());
            f.render_widget(ratatui::widgets::Clear, area);
            let msg = if force {
                "Are you sure you want to FORCE PUSH this project?\nThis action is destructive!\n\n(y/Enter: Yes, n/Esc: No)"
            } else {
                "Are you sure you want to PUSH this project?\n\n(y/Enter: Yes, n/Esc: No)"
            };
            let block = Block::default().title("Confirm Push").borders(Borders::ALL).border_style(Style::default().fg(if force { Color::Red } else { Color::Yellow }));
            let p = Paragraph::new(msg).block(block).alignment(ratatui::layout::Alignment::Center);
            f.render_widget(p, area);
        }
    }

    if let Some(msg) = &app.flash_message {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default().title("Message (Press any key to dismiss)").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan));
        let p = Paragraph::new(msg.as_str()).block(block).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, area);
    }
}

fn render_file_explorer(f: &mut Frame, app: &mut App, area: Rect) {
    let mut items = Vec::new();
    for entry in &app.file_explorer.entries {
        let (style, prefix) = if entry.is_git_repo {
            (Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD), "[GIT] ")
        } else if entry.is_dir {
            (Style::default().fg(Color::LightBlue), "[DIR] ")
        } else {
            (Style::default().fg(Color::DarkGray), "[FILE] ")
        };
        
        let mut spans = vec![
            Span::styled(prefix, style),
            Span::raw(&entry.name),
        ];
        
        if let Some(info) = &entry.git_info {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(info.clone(), Style::default().fg(Color::DarkGray)));
        }
        
        items.push(ListItem::new(Line::from(spans)));
    }
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(format!(" File Explorer: {} ", app.file_explorer.current_path.display()));
        
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
        
    f.render_stateful_widget(list, area, &mut app.file_explorer.list_state);
}

fn render_projects_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .projects
        .iter()
        .map(|p| {
            let checkbox = if p.enabled { "[x]" } else { "[ ]" };
            let style = if p.enabled { Style::default() } else { Style::default().fg(Color::DarkGray) };
            
            let commit_count = if p.enabled {
                let count: usize = p.branches.iter().map(|b| b.commits.len()).sum();
                format!("({} commits)", count)
            } else {
                "(disabled)".to_string()
            };
            
            let line = Line::from(vec![
                Span::styled(format!("{} ", checkbox), style),
                Span::styled(format!("{} ", p.name), style),
                Span::styled(commit_count, style),
            ]);
            
            ListItem::new(vec![line])
        })
        .collect();

    let title = if app.is_loading { "Projects [LOADING...]" } else { "Projects" };
    let mut block = Block::default().borders(Borders::ALL).title(title);
    if let AppMode::Normal = app.mode {
        block = block.border_style(Style::default().fg(Color::Yellow));
    }

    let items = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, area, &mut app.project_list_state);
}

fn render_commits_list(f: &mut Frame, app: &mut App, area: Rect) {
    let mut items = Vec::new();

    let mut projects_to_show = Vec::new();
    
    if let AppMode::Details = app.mode {
        if let Some(idx) = app.selected_project_idx {
            if idx < app.projects.len() {
                projects_to_show.push(&app.projects[idx]);
            }
        }
    } else {
        for proj in &app.projects {
            if proj.enabled {
                projects_to_show.push(proj);
            }
        }
    }

    let mut commit_count = 0;
    const LIMIT: usize = 1000;
    let mut total_commits_in_view = 0;

    for proj in projects_to_show {
        if proj.branches.is_empty() {
            continue;
        }
        
        let proj_commits: usize = proj.branches.iter().map(|b| b.commits.len()).sum();
        total_commits_in_view += proj_commits;
        
        if commit_count >= LIMIT {
            continue; // Skip rendering more projects if we hit the limit
        }
        
        if !proj.is_expanded {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("Project: {} (Collapsed)", proj.name), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            ])));
            continue;
        }

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("Project: {} ", proj.name), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])));
        
        for branch in &proj.branches {
            if commit_count >= LIMIT {
                break;
            }
            
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("Branch: {} ", branch.name), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ])));
            
            let mut last_date = String::new();
            let mut last_author = String::new();
            
            for commit in &branch.commits {
                if commit_count >= LIMIT {
                    break;
                }
                
                let current_date = commit.date.format("%Y-%m-%d").to_string();
                if current_date != last_date {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("Date: {} ", current_date), Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
                    ])));
                    last_date = current_date;
                    last_author = String::new();
                }

                let current_author = &commit.author;
                if current_author != &last_author {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(format!("Author: {} ", current_author), Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                    ])));
                    last_author = current_author.clone();
                }
                
                let push_status = if commit.is_pushed {
                    Span::styled("[Pushed] ", Style::default().fg(Color::Green))
                } else {
                    Span::styled("[Unpushed] ", Style::default().fg(Color::Red))
                };
                
                let line = Line::from(vec![
                    Span::raw("        "),
                    push_status,
                    Span::raw(format!("{} ", commit.id.chars().take(7).collect::<String>())),
                    Span::styled(format!("{} ", commit.date.format("%H:%M")), Style::default().fg(Color::Blue)),
                    Span::raw(format!(" {}", commit.message)),
                ]);
                items.push(ListItem::new(vec![line]));
                commit_count += 1;
            }
        }
    }
    
    if total_commits_in_view > LIMIT {
        let hidden = total_commits_in_view - LIMIT;
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("... and {} more commits (export to see all)", hidden), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ])));
    }

    let title = if app.is_loading { "Commits [LOADING...]" } else { "Commits" };
    let mut block = Block::default().borders(Borders::ALL).title(title);
    if let AppMode::Details = app.mode {
        block = block.border_style(Style::default().fg(Color::Yellow));
    }

    let items = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, area, &mut app.commit_list_state);
}

fn render_author_autocomplete(f: &mut Frame, app: &mut App, area: Rect) {
    let filtered = app.get_filtered_authors();
    let items: Vec<ListItem> = filtered.into_iter().map(ListItem::new).collect();
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title("Known Authors (from latest commits)");
        
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, area, &mut app.author_list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
