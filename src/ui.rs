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
    let date_display = if app.date_start_filter == app.date_end_filter {
        app.date_start_filter.format("%Y-%m-%d").to_string()
    } else {
        format!("{}..{}", app.date_start_filter.format("%Y-%m-%d"), app.date_end_filter.format("%Y-%m-%d"))
    };
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
                let footer = Paragraph::new("Esc: Cancel | j/k: Navigate | l/Enter: Enter Dir | h/Backspace: Go Up | s: Add as Source | /: Search")
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
        AppMode::Normal | AppMode::Details => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let help_text = match app.mode {
                AppMode::Normal => "q: Quit | P: Profile | a: Author | d: Date | p: Add Path | l/Enter: Commits | Space: Toggle | r: Remove | e: Export",
                AppMode::Details => "q: Quit | h/Esc: Back | j/k: Navigate Commits | e: Export",
                _ => "",
            };
            let footer = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        }
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
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(&entry.name),
        ])));
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

    let mut block = Block::default().borders(Borders::ALL).title("Projects");
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

    if let Some(idx) = app.selected_project_idx
        && idx < app.projects.len() {
            for branch in &app.projects[idx].branches {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("Branch: {} ", branch.name), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                ])));
                
                for commit in &branch.commits {
                    let push_status = if commit.is_pushed {
                        Span::styled("[Pushed] ", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("[Unpushed] ", Style::default().fg(Color::Red))
                    };
                    
                    let line = Line::from(vec![
                        Span::raw("  "),
                        push_status,
                        Span::raw(format!("{} ", commit.id.chars().take(7).collect::<String>())),
                        Span::styled(format!("{} ", commit.date.format("%H:%M")), Style::default().fg(Color::Blue)),
                        Span::styled(format!(" [{}] ", commit.author), Style::default().fg(Color::Cyan)),
                        Span::raw(&commit.message),
                    ]);
                    items.push(ListItem::new(vec![line]));
                }
            }
        }

    let mut block = Block::default().borders(Borders::ALL).title("Commits");
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
