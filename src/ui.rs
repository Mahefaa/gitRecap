use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
        Span::raw(format!("| Projects w/ Commits: {} | ", app.projects.iter().filter(|p| p.enabled && !p.dates.is_empty()).count())),
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
        AppMode::InputBranch => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let input_widget = Paragraph::new(app.input.value())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Enter Branches separated by commas (e.g. main,dev) (Esc cancel, Enter submit)"));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 1 + app.input.visual_cursor() as u16,
                chunks[2].y + 1,
            ));
        },
        AppMode::InputCommitSearch => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let input_widget = Paragraph::new(app.input.value())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Search Commits by message or ID (Esc cancel, Enter submit)"));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 1 + app.input.visual_cursor() as u16,
                chunks[2].y + 1,
            ));
        },
        AppMode::Command => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let input_widget = Paragraph::new(format!(":{}", app.input.value()))
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Command (e.g. cp, cd, cb) (Esc cancel, Enter submit)"));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 2 + app.input.visual_cursor() as u16,
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
        AppMode::Normal | AppMode::Details | AppMode::CommitsView | AppMode::ConfirmQuit | AppMode::Help => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            app.projects_area = Some(main_chunks[0]);
            app.commits_area = Some(main_chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let help_text = match app.mode {
                AppMode::Normal | AppMode::ConfirmQuit | AppMode::Help => "?: Help Modal | q: Quit | j/k: Nav | g/G: Top/Bot | Space: Toggle | c: Collapse | :cmd | P: Profile | p: Explorer | u: Push | R: Refresh",
                AppMode::Details => "q: Quit | h/Esc/Alt+Left: Back | j/k: Nav | g/G: Top/Bot | c: Collapse | :cmd | /: Search | e: Export | ?: Help",
                AppMode::CommitsView => "q: Quit | h/Esc/Alt+Left: Back | j/k: Nav | g/G: Top/Bot | c: Collapse | :cmd | /: Search | e: Export | ?: Help",
                _ => "",
            };
            let footer = Paragraph::new(help_text)
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL));
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

    if let AppMode::ConfirmQuit = app.mode {
        let block = Block::default().borders(Borders::ALL).title("Confirm Quit").border_style(Style::default().fg(Color::Red));
        let area = centered_rect(30, 20, chunks[1]);
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(
            Paragraph::new("Are you sure you want to quit? [y/N]")
                .alignment(ratatui::layout::Alignment::Center)
                .block(block),
            area,
        );
    }

    if let AppMode::Help = app.mode {
        let help_text = vec![
            Line::from(Span::styled("--- Global Navigation ---", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(" j/k, Up/Down  : Navigate lists"),
            Line::from(" g / G         : Jump to Top / Bottom"),
            Line::from(" q             : Quit application"),
            Line::from(" ?             : Toggle this help modal"),
            Line::from(""),
            Line::from(Span::styled("--- Projects View ---", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(" Space         : Toggle project enable/disable"),
            Line::from(" s             : Toggle all projects"),
            Line::from(" c             : Collapse/Expand project node"),
            Line::from(" l / Enter     : View commit details / expand"),
            Line::from(" r             : Remove selected project from list"),
            Line::from(" D             : Remove all projects from list"),
            Line::from(" H             : Hide projects with 0 commits"),
            Line::from(""),
            Line::from(Span::styled("--- Filtering & Profiles ---", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(" a / d / b     : Filter by Author / Date / Branch"),
            Line::from(" P             : Switch or Create Profile"),
            Line::from(" p             : Open File Explorer (Add sources visually)"),
            Line::from(" A             : Add a source path manually"),
            Line::from(" R             : Reload Git Data"),
            Line::from(""),
            Line::from(Span::styled("--- Commits View ---", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(" h / Esc / Alt+Left : Go back to Projects view"),
            Line::from(" /             : Search inside commits"),
            Line::from(" c             : Collapse/Expand Date or Branch"),
            Line::from(" e             : Export summary to summary.txt"),
            Line::from(" u / U         : Push / Force Push selected project"),
            Line::from(""),
            Line::from(Span::styled("--- Command Mode (:) ---", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(" :cp / :ep     : Collapse / Expand all projects"),
            Line::from(" :cd / :ed     : Collapse / Expand all dates"),
            Line::from(" :cb / :eb     : Collapse / Expand all branches"),
            Line::from(""),
            Line::from(Span::styled("Press '?' or 'Esc' to close this modal.", Style::default().fg(Color::DarkGray))),
        ];
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Help & Shortcuts")
            .border_style(Style::default().fg(Color::Green));
            
        let area = centered_rect(60, 80, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(Paragraph::new(help_text).block(block), area);
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
        .filter(|p| {
            if app.hide_zero_commits {
                !p.dates.is_empty()
            } else {
                true
            }
        })
        .map(|p| {
            let checkbox = if p.enabled { "[x]" } else { "[ ]" };
            let collapse_icon = if p.is_expanded { "[-]" } else { "[+]" };
            let style = if p.enabled { Style::default() } else { Style::default().fg(Color::DarkGray) };
            
            let commit_count = if p.enabled {
                let count: usize = p.dates.iter().flat_map(|d| d.branches.iter().map(|b| b.commits.len())).sum();
                if count > 0 {
                    format!("({} commits)", count)
                } else {
                    crate::git_utils::get_last_commit_info(&p.path).unwrap_or_else(|| "(0 commits)".to_string())
                }
            } else {
                "(disabled)".to_string()
            };
            
            let line = Line::from(vec![
                Span::styled(format!("{} {} ", checkbox, collapse_icon), style),
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

    let is_active = matches!(app.mode, AppMode::Normal);
    let hl_style = if is_active {
        Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let hl_symbol = if is_active { ">> " } else { "   " };

    let items = List::new(items)
        .block(block)
        .highlight_style(hl_style)
        .highlight_symbol(hl_symbol);

    f.render_stateful_widget(items, area, &mut app.project_list_state);
}

fn render_commits_list(f: &mut Frame, app: &mut App, area: Rect) {
    let mut items = Vec::new();
    app.commit_list_map.clear();

    let mut projects_to_show = Vec::new();
    
    if let AppMode::Details = app.mode {
        if let Some(idx) = app.selected_project_idx {
            if idx < app.projects.len() {
                projects_to_show.push((idx, &app.projects[idx]));
            }
        }
    } else {
        for (idx, proj) in app.projects.iter().enumerate() {
            if proj.enabled {
                projects_to_show.push((idx, proj));
            }
        }
    }

    let mut commit_count = 0;
    const LIMIT: usize = 1000;
    let mut total_commits_in_view = 0;
    
    let search_lower = if app.commit_search.is_empty() {
        None
    } else {
        Some(app.commit_search.to_lowercase())
    };

    for (proj_idx, proj) in projects_to_show {
        if proj.dates.is_empty() {
            continue;
        }
        
        let proj_commits: usize = proj.dates.iter().flat_map(|d| d.branches.iter().map(|b| b.commits.len())).sum();
        total_commits_in_view += proj_commits;
        
        if commit_count >= LIMIT {
            continue;
        }
        
        if !proj.is_expanded {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{} (collapsed)", proj.name), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            ])));
            app.commit_list_map.push((proj_idx, None, None));
            continue;
        }

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", proj.name), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])));
        app.commit_list_map.push((proj_idx, None, None));
        
        for (date_idx, date_group) in proj.dates.iter().enumerate() {
            if commit_count >= LIMIT { break; }
            if !date_group.is_expanded {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{} (collapsed)", date_group.date), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                ])));
                app.commit_list_map.push((proj_idx, Some(date_idx), None));
                continue;
            }
            
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{} ", date_group.date), Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
            ])));
            app.commit_list_map.push((proj_idx, Some(date_idx), None));
            
            for (branch_idx, branch_group) in date_group.branches.iter().enumerate() {
                if commit_count >= LIMIT { break; }
                if !branch_group.is_expanded {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("{} (collapsed)", branch_group.name), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    ])));
                    app.commit_list_map.push((proj_idx, Some(date_idx), Some(branch_idx)));
                    continue;
                }
                
                if !branch_group.commits.is_empty() {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("{} ", branch_group.name), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    ])));
                    app.commit_list_map.push((proj_idx, Some(date_idx), Some(branch_idx)));
                }
                
                for commit in &branch_group.commits {
                    if let Some(search) = &search_lower {
                        if !commit.message.to_lowercase().contains(search) && !commit.id.to_lowercase().contains(search) {
                            continue;
                        }
                    }

                    if commit_count >= LIMIT { break; }
                    
                    let push_status = if commit.is_pushed {
                        Span::raw("")
                    } else {
                        Span::styled("* ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    };
                    
                    let line = Line::from(vec![
                        Span::raw("      "),
                        push_status,
                        Span::raw(format!("{} ", commit.id.chars().take(7).collect::<String>())),
                        Span::styled(format!("{} ", commit.date.format("%H:%M")), Style::default().fg(Color::Blue)),
                        Span::styled(format!("[{}] ", commit.author), Style::default().fg(Color::LightCyan)),
                        Span::raw(format!("{}", commit.message)),
                    ]);
                    items.push(ListItem::new(vec![line]));
                    app.commit_list_map.push((proj_idx, Some(date_idx), Some(branch_idx)));
                    commit_count += 1;
                }
            }
        }
    }
    
    if total_commits_in_view > LIMIT {
        let hidden = total_commits_in_view - LIMIT;
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("... and {} more commits (export to see all)", hidden), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ])));
        app.commit_list_map.push((usize::MAX, None, None));
    }

    let title = if app.is_loading { "Commits [LOADING...]" } else { "Commits" };
    let mut title_with_search = title.to_string();
    if !app.commit_search.is_empty() {
        title_with_search = format!("{} (Search: {})", title, app.commit_search);
    }
    
    let mut block = Block::default().borders(Borders::ALL).title(title_with_search);
    if matches!(app.mode, AppMode::Details | AppMode::CommitsView) {
        block = block.border_style(Style::default().fg(Color::Yellow));
    }

    let is_active = matches!(app.mode, AppMode::Details | AppMode::CommitsView);
    let hl_style = if is_active {
        Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let hl_symbol = if is_active { ">> " } else { "   " };

    let items = List::new(items)
        .block(block)
        .highlight_style(hl_style)
        .highlight_symbol(hl_symbol);

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
