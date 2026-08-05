use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
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

    if !app.config.no_prank {
        let text = "YOU'RE GAY ".repeat(200);
        let bg = Paragraph::new(text)
            .style(Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: false });
        f.render_widget(bg, f.area());
    }

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
        AppMode::InputDate | AppMode::InputBranch | AppMode::InputExportPath => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let title = match app.mode {
                AppMode::InputDate => "Enter Date [YYYY-MM-DD] or Range [YYYY-MM-DD..YYYY-MM-DD] (Esc cancel, Enter submit)",
                AppMode::InputBranch => "Enter Branches separated by commas (e.g. main,dev) (Esc cancel, Enter submit)",
                AppMode::InputExportPath => "Enter Export File Path (Absolute or relative, Esc cancel, Enter submit)",
                _ => "",
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
        AppMode::Search(target) => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);
                
            render_projects_list(f, app, main_chunks[0]);
            render_commits_list(f, app, main_chunks[1]);
            
            let title = match target {
                crate::app::SearchTarget::Projects => "Search Projects (Esc cancel, Enter submit)",
                crate::app::SearchTarget::Commits => "Search Commits (Esc cancel, Enter submit)",
            };
            
            let input_widget = Paragraph::new(format!("/{}", app.input.value()))
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(title));
            f.render_widget(input_widget, chunks[2]);
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position((
                chunks[2].x + 2 + app.input.visual_cursor() as u16,
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
            
            if let Some(diff) = app.diff_content.clone() {
                let commit_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(main_chunks[1]);
                
                render_commits_list(f, app, commit_chunks[0]);
                
                let diff_block = Block::default().title("Diff Viewer (PgUp/PgDown to scroll)").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan));
                
                let p = Paragraph::new(diff).block(diff_block).scroll((app.diff_scroll, 0));
                f.render_widget(p, commit_chunks[1]);
            } else {
                render_commits_list(f, app, main_chunks[1]);
            }
            
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
        AppMode::Dashboard => {
            let mut time_per_project: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
            let mut commits_per_project: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            let mut total_commits = 0;
            let mut total_hours = 0.0;
            
            for d in &app.timeline {
                for p in &d.projects {
                    let mut proj_commits = 0;
                    let mut proj_hours = 0.0;
                    for a in &p.authors {
                        proj_commits += a.branches.iter().map(|b| b.commits.len()).sum::<usize>();
                        
                        let mut dates: Vec<_> = a.branches.iter().flat_map(|b| b.commits.iter().map(|c| c.date)).collect();
                        dates.sort();
                        let mut prev_date: Option<chrono::DateTime<chrono::Local>> = None;
                        for date in dates {
                            let mut hours = 0.5; // Base session time
                            if let Some(prev) = prev_date {
                                let diff = (date - prev).num_minutes();
                                if diff <= 120 && diff >= 0 {
                                    hours = diff as f64 / 60.0;
                                }
                            }
                            proj_hours += hours;
                            prev_date = Some(date);
                        }
                    }
                    
                    total_commits += proj_commits;
                    
                    if proj_commits > 0 {
                        *commits_per_project.entry(p.name.clone()).or_insert(0) += proj_commits;
                        *time_per_project.entry(p.name.clone()).or_insert(0.0) += proj_hours;
                        total_hours += proj_hours;
                    }
                }
            }
            
            let mut project_stats: Vec<(&String, &f64)> = time_per_project.iter().collect();
            project_stats.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            let row_count = project_stats.len() + 1; // +1 for "All Projects"
            let sel = match app.dashboard_list_state.selected() {
                Some(i) => if i >= row_count { row_count.saturating_sub(1) } else { i },
                None => { app.dashboard_list_state.select(Some(0)); 0 }
            };
            if app.dashboard_list_state.selected() != Some(sel) {
                app.dashboard_list_state.select(Some(sel));
            }
            
            let selected_project_name = if sel == 0 {
                None
            } else {
                Some(project_stats[sel - 1].0.clone())
            };
            
            let colors = [Color::Red, Color::Yellow, Color::Green, Color::Cyan, Color::Blue, Color::Magenta, Color::LightRed, Color::LightCyan, Color::LightYellow];
            let mut proj_colors: std::collections::HashMap<String, Color> = std::collections::HashMap::new();
            for (i, (p_name, _)) in project_stats.iter().enumerate() {
                proj_colors.insert((*p_name).clone(), colors[i % colors.len()]);
            }
            
            let mut grouped_data: std::collections::HashMap<String, std::collections::HashMap<String, f64>> = std::collections::HashMap::new();
            
            for d in &app.timeline {
                for p in &d.projects {
                    for a in &p.authors {
                        let mut all_commits: Vec<_> = a.branches.iter().flat_map(|b| b.commits.iter()).collect();
                        all_commits.sort_by(|x, y| x.date.cmp(&y.date));
                        let mut prev_date: Option<chrono::DateTime<chrono::Local>> = None;
                        for c in all_commits {
                            let mut hours = 0.5;
                            if let Some(prev) = prev_date {
                                let diff = (c.date - prev).num_minutes();
                                if diff <= 120 && diff >= 0 {
                                    hours = diff as f64 / 60.0;
                                }
                            }
                            prev_date = Some(c.date);
                            
                            let key = match app.dashboard_resolution {
                                crate::app::TimeResolution::Day => c.date.format("%Y-%m-%d").to_string(),
                                crate::app::TimeResolution::Week => c.date.format("%Y-W%W").to_string(),
                                crate::app::TimeResolution::Month => c.date.format("%Y-%m").to_string(),
                            };
                            
                            *grouped_data.entry(key).or_default().entry(p.name.clone()).or_insert(0.0) += hours;
                        }
                    }
                }
            }
            
            let mut sorted_keys: Vec<String> = grouped_data.keys().cloned().collect();
            sorted_keys.sort();
            let display_keys: Vec<String> = sorted_keys.into_iter().rev().take(15).rev().collect();
            
            use ratatui::widgets::{BarGroup, Bar};
            let mut bar_groups: Vec<ratatui::widgets::BarGroup> = Vec::new();
            for key in &display_keys {
                let p_map = &grouped_data[key];
                let total_hours: f64 = p_map.values().sum();
                let mut bars = Vec::new();
                for (p_name, _) in &project_stats {
                    if let Some(ref sp) = selected_project_name {
                        if **p_name != *sp { continue; } // Filter rendering dynamically so % is still global
                    }
                    if let Some(&hours) = p_map.get(*p_name) {
                        let color = proj_colors.get(*p_name).copied().unwrap_or(Color::Green);
                        let percent = if total_hours > 0.0 { (hours / total_hours) * 100.0 } else { 0.0 };
                        let percent_rounded = percent.round() as u64;
                        let bar_value = (hours * 10.0).round() as u64;
                        // Minimum height of 1 to render the text if > 0
                        let bar_value = if hours > 0.0 && bar_value == 0 { 1 } else { bar_value };
                        bars.push(Bar::default().value(bar_value).text_value(format!("{}%", percent_rounded)).style(Style::default().fg(color)));
                    }
                }
                if !bars.is_empty() {
                    let display_label = match app.dashboard_resolution {
                        crate::app::TimeResolution::Day => if key.len() >= 10 { &key[5..10] } else { key.as_str() }, // "07-10"
                        crate::app::TimeResolution::Week => if key.len() >= 4 { &key[2..] } else { key.as_str() }, // "26-W30"
                        crate::app::TimeResolution::Month => if key.len() >= 7 { &key[2..7] } else { key.as_str() }, // "26-07"
                    };
                    bar_groups.push(BarGroup::default().label(Line::from(display_label)).bars(&bars));
                }
            }
            
            let mut most_active_repo = String::from("None");
            let mut max_repo_commits = 0;
            if let Some((best_repo, &max_commits)) = commits_per_project.iter().max_by_key(|(_, v)| *v) {
                most_active_repo = best_repo.clone();
                max_repo_commits = max_commits;
            }
            
            let dashboard_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);
                
            let stats_text = vec![
                Line::from(vec![
                    Span::styled(format!("Total Commits in View: {} | ", total_commits), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("Total Time Spent: {:.1}h ({:.2}d) | ", total_hours, total_hours / app.config.working_day_hours), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("Most Active Repository: {} ({} commits)", most_active_repo, max_repo_commits), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(Span::styled("This dashboard visualizes your Git activity across all enabled projects over time.", Style::default().fg(Color::DarkGray))),
                Line::from(Span::styled(format!("Working day configured as: {:.1} hours", app.config.working_day_hours), Style::default().fg(Color::DarkGray))),
            ];
            
            let stats_block = Paragraph::new(stats_text)
                .block(Block::default().borders(Borders::ALL).title("Insights (Press 'Esc' or 'v' to exit)").border_style(Style::default().fg(Color::Green)))
                .alignment(ratatui::layout::Alignment::Center);
                
            f.render_widget(stats_block, dashboard_layout[0]);
            
            // Pie Chart / Time Breakdown Table
            use ratatui::widgets::{Table, Row, Cell};
            let mut table_rows = Vec::new();
            
            table_rows.push(Row::new(vec![
                Cell::from("★ ALL PROJECTS").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Cell::from(total_commits.to_string()),
                Cell::from(format!("{:.1}h", total_hours)),
                Cell::from(format!("{:.2}d", total_hours / app.config.working_day_hours)),
                Cell::from("100.0%"),
            ]));
            
            for (p_name, &p_hours) in project_stats {
                let p_commits = commits_per_project.get(p_name).unwrap_or(&0);
                let p_days = p_hours / app.config.working_day_hours;
                let percent = if total_hours > 0.0 { (p_hours / total_hours) * 100.0 } else { 0.0 };
                
                table_rows.push(Row::new(vec![
                    Cell::from(p_name.clone()).style(Style::default().fg(Color::Yellow)),
                    Cell::from(p_commits.to_string()),
                    Cell::from(format!("{:.1}h", p_hours)).style(Style::default().fg(Color::Cyan)),
                    Cell::from(format!("{:.2}d", p_days)).style(Style::default().fg(Color::Magenta)),
                    Cell::from(format!("{:.1}%", percent)).style(Style::default().fg(Color::Green)),
                ]));
            }
            
            let table = Table::new(table_rows, [
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ])
            .header(Row::new(vec!["Project (Time Breakdown)", "Commits", "Hours Spent", "Days Spent", "% of Total"]).style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)))
            .block(Block::default().borders(Borders::ALL).title("Time Spent Breakdown (Use j/k to filter chart)").border_style(Style::default().fg(Color::Blue)))
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD))
            .highlight_symbol(">> ");
            
            f.render_stateful_widget(table, dashboard_layout[1], &mut app.dashboard_list_state);
            
            let barchart_title = match app.dashboard_resolution {
                crate::app::TimeResolution::Day => "Time Tracking % (Daily) | 't' to toggle resolution",
                crate::app::TimeResolution::Week => "Time Tracking % (Weekly) | 't' to toggle resolution",
                crate::app::TimeResolution::Month => "Time Tracking % (Monthly) | 't' to toggle resolution",
            };
            
            let barchart_block_title = if let Some(ref sp) = selected_project_name {
                format!("{} for '{}'", barchart_title, sp)
            } else {
                format!("Global {}", barchart_title)
            };
            
            // Note: .data(bar_groups) passes a single reference or takes ownership depending on Ratatui version.
            // Using .data(bar_groups) passes the iterator of BarGroups.
            // But wait, the BarChart's .data() for BarGroup needs a reference sometimes. 
            // In ratatui 0.25+, `BarGroup` implements the trait properly.
            // If .data() expects `&'a [(&'a str, u64)]`, we cannot pass `Vec<BarGroup>`.
            // Wait, ratatui's BarChart has a `data()` method, but does it accept `BarGroup` directly? 
            // Yes, ratatui 0.26+ supports `.data(BarGroup)` directly if it's the `ratatui::widgets::BarChart::default().data(&display_data)`?
            // Actually, `.data()` on BarChart has a generic bound.
            // Let's use `.data(ratatui::widgets::BarGroup)`. Wait, we just pass the vector:
            // `.data(ratatui::widgets::BarChart::default().data(bar_groups))` might not work if lifetimes don't match.
            // But wait, ratatui's `BarGroup` expects a reference! Let's pass `&bar_groups`.
            
            // Wait, `data` takes `impl IntoIterator<Item = BarGroup>`.
            let mut barchart = ratatui::widgets::BarChart::default()
                .block(Block::default().title(barchart_block_title).borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)))
                .bar_width(4) // slightly wider than 3 for better visibility
                .bar_gap(1)
                .group_gap(6) // prevents even the shortened labels from overlapping
                .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
            
            for group in bar_groups {
                barchart = barchart.data(group);
            }
                
            f.render_widget(barchart, dashboard_layout[2]);
        }
    }

    if let AppMode::ConfirmQuit = app.mode {
        let block = Block::default().borders(Borders::ALL).title("Confirm Quit").border_style(Style::default().fg(Color::Red));
        let area = centered_rect(30, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        
        let pad_top = inner_area.height.saturating_sub(1) / 2;
        let text_area = ratatui::layout::Rect {
            x: inner_area.x,
            y: inner_area.y + pad_top,
            width: inner_area.width,
            height: 1,
        };
        
        f.render_widget(
            Paragraph::new("Are you sure you want to quit? [y/N]")
                .alignment(ratatui::layout::Alignment::Center),
            text_area,
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
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        
        let text_height = msg.lines().count() as u16;
        let pad_top = inner_area.height.saturating_sub(text_height) / 2;
        let text_area = ratatui::layout::Rect {
            x: inner_area.x,
            y: inner_area.y + pad_top,
            width: inner_area.width,
            height: text_height,
        };
        
        let p = Paragraph::new(msg.as_str()).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, text_area);
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
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD))
        .highlight_symbol(">> ");
        
    f.render_stateful_widget(list, area, &mut app.file_explorer.list_state);
}

fn render_projects_list(f: &mut Frame, app: &mut App, area: Rect) {
    if app.visible_projects.is_empty() && app.projects.is_empty() {
        // Fallback for startup
        app.update_visible_projects();
    }
    let items: Vec<ListItem> = app
        .visible_projects
        .iter()
        .map(|&idx| &app.projects[idx])
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
                    p.last_commit_info.clone()
                }
            } else {
                "(disabled)".to_string()
            };
            
            let pride = if app.config.no_prank { Color::Reset } else {
                let colors = [Color::Red, Color::LightRed, Color::Yellow, Color::LightGreen, Color::Green, Color::LightCyan, Color::Cyan, Color::LightBlue, Color::Blue, Color::LightMagenta, Color::Magenta];
                colors[p.name.len() % colors.len()]
            };
            
            let name_style = if app.config.no_prank { style } else { style.fg(pride) };
            
            let line = Line::from(vec![
                Span::styled(format!("{} {} ", checkbox, collapse_icon), style),
                Span::styled(format!("{} ", p.name), name_style),
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
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
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

    let target_proj = if let AppMode::Details = app.mode {
        if let Some(idx) = app.selected_project_idx {
            if idx < app.projects.len() {
                Some(app.projects[idx].name.clone())
            } else { None }
        } else { None }
    } else { None };

    let mut commit_count = 0;
    const LIMIT: usize = 1000;
    let mut total_commits_in_view = 0;
    
    for (date_idx, date_group) in app.timeline.iter().enumerate() {
        let mut visible_projects = Vec::new();
        for (proj_idx, proj) in date_group.projects.iter().enumerate() {
            if let Some(ref target) = target_proj {
                if proj.name != *target { continue; }
            }
            visible_projects.push((proj_idx, proj));
        }
        
        if visible_projects.is_empty() { continue; }
        
        let date_commits: usize = visible_projects.iter()
            .flat_map(|(_, p)| p.authors.iter().flat_map(|a| a.branches.iter().map(|b| b.commits.len())))
            .sum();
            
        total_commits_in_view += date_commits;
        
        if commit_count >= LIMIT { continue; }
        
        if !date_group.is_expanded {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{} (collapsed)", date_group.date), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            ])));
            app.commit_list_map.push((date_idx, None, None, None, None));
            continue;
        }

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", date_group.date), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])));
        app.commit_list_map.push((date_idx, None, None, None, None));
        
        for (proj_idx, proj) in visible_projects {
            if commit_count >= LIMIT { break; }
            if !proj.is_expanded {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{} (collapsed)", proj.name), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                ])));
                app.commit_list_map.push((date_idx, Some(proj_idx), None, None, None));
                continue;
            }
            
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{} ", proj.name), Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
            ])));
            app.commit_list_map.push((date_idx, Some(proj_idx), None, None, None));
            
            for (author_idx, author_group) in proj.authors.iter().enumerate() {
                if commit_count >= LIMIT { break; }
                if !author_group.is_expanded {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("{} (collapsed)", author_group.name), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    ])));
                    app.commit_list_map.push((date_idx, Some(proj_idx), Some(author_idx), None, None));
                    continue;
                }
                
                if !author_group.branches.is_empty() {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("{} ", author_group.name), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    ])));
                    app.commit_list_map.push((date_idx, Some(proj_idx), Some(author_idx), None, None));
                }
                
                for (branch_idx, branch_group) in author_group.branches.iter().enumerate() {
                    if commit_count >= LIMIT { break; }
                    if !branch_group.is_expanded {
                        items.push(ListItem::new(Line::from(vec![
                            Span::raw("      "),
                            Span::styled(format!("{} (collapsed)", branch_group.name), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                        ])));
                        app.commit_list_map.push((date_idx, Some(proj_idx), Some(author_idx), Some(branch_idx), None));
                        continue;
                    }
                    
                    if !branch_group.commits.is_empty() {
                        items.push(ListItem::new(Line::from(vec![
                            Span::raw("      "),
                            Span::styled(format!("{} ", branch_group.name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        ])));
                        app.commit_list_map.push((date_idx, Some(proj_idx), Some(author_idx), Some(branch_idx), None));
                    }
                    
                    for (commit_idx, commit) in branch_group.commits.iter().enumerate() {
                        if commit_count >= LIMIT { break; }
                        
                        let push_status = if commit.is_pushed {
                            Span::raw("")
                        } else {
                            Span::styled("* ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        };
                        
                        let pride = if app.config.no_prank { Color::Reset } else {
                            let colors = [Color::Red, Color::Yellow, Color::Green, Color::Cyan, Color::Blue, Color::Magenta];
                            colors[commit_idx % colors.len()]
                        };
                        
                        let id_style = if app.config.no_prank { Style::default() } else { Style::default().fg(pride) };
                        let date_style = if app.config.no_prank { Style::default().fg(Color::Blue) } else { Style::default().fg(pride) };
                        let msg_style = if app.config.no_prank { Style::default() } else { Style::default().fg(pride) };
    
                        let line = Line::from(vec![
                            Span::raw("        "),
                            push_status,
                            Span::styled(format!("{} ", commit.id.chars().take(7).collect::<String>()), id_style),
                            Span::styled(format!("{} ", commit.date.format("%H:%M")), date_style),
                            Span::styled(format!("{}", commit.message), msg_style),
                        ]);
                        items.push(ListItem::new(vec![line]));
                        app.commit_list_map.push((date_idx, Some(proj_idx), Some(author_idx), Some(branch_idx), Some(commit_idx)));
                        commit_count += 1;
                    }
                }
            }
        }
    }
    
    if total_commits_in_view > LIMIT {
        let hidden = total_commits_in_view - LIMIT;
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("... and {} more commits (export to see all)", hidden), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ])));
        app.commit_list_map.push((usize::MAX, None, None, None, None));
    }

    let title = if app.is_loading { "Commits [LOADING...]" } else { "Commits" };
    let title_with_search = title.to_string();
    
    let mut block = Block::default().borders(Borders::ALL).title(title_with_search);
    if matches!(app.mode, AppMode::Details | AppMode::CommitsView) {
        block = block.border_style(Style::default().fg(Color::Yellow));
    }

    let is_active = matches!(app.mode, AppMode::Details | AppMode::CommitsView);
    let hl_style = if is_active {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
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
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD))
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
