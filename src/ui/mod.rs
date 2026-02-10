use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal as RatatuiTerminal,
};
use std::io;
use textwrap::wrap;

// Import types needed for the UI logic
use crate::core::decisions::Choice;
use crate::core::types::{DecisionImpact, RiskVector};

/// RAII Terminal wrapper - ensures cleanup on drop
pub struct Terminal {
    terminal: RatatuiTerminal<CrosstermBackend<io::Stdout>>,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = RatatuiTerminal::new(backend)?;

        Ok(Self { terminal })
    }

    pub fn width(&self) -> usize {
        self.terminal.size().map(|s| s.width as usize).unwrap_or(80)
    }

    pub fn height(&self) -> usize {
        self.terminal
            .size()
            .map(|s| s.height as usize)
            .unwrap_or(24)
    }

    /// Draw a frame with the given render function
    fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    /// Clear the screen
    pub fn clear(&mut self) -> io::Result<()> {
        self.terminal.clear()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Always cleanup, even on panic
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

/// Wait for Enter key press with proper event filtering
pub fn wait_for_enter() -> io::Result<()> {
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Enter,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            return Ok(());
        }
    }
}

/// Display paginated text with proper scrolling
pub fn display_paginated_text(text: &str, term: &mut Terminal) -> io::Result<()> {
    let mut scroll: u16 = 0;

    loop {
        let size = term.terminal.size()?;
        let max_scroll = text.lines().count().saturating_sub(size.height as usize - 4);

        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(f.area());

            // Content area
            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
                .scroll((scroll, 0))
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[0]);

            // Help text
            let help_text = if scroll < max_scroll as u16 {
                "↑↓ to scroll | Enter to continue | q to quit"
            } else {
                "Enter to continue | q to quit"
            };

            let help = Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));

            f.render_widget(help, chunks[1]);
        })?;

        // Handle input
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                scroll = scroll.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if (scroll as usize) < max_scroll {
                    scroll += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::PageUp,
                kind: KeyEventKind::Press,
                ..
            }) => {
                scroll = scroll.saturating_sub(10);
            }
            Event::Key(KeyEvent {
                code: KeyCode::PageDown,
                kind: KeyEventKind::Press,
                ..
            }) => {
                scroll = (scroll + 10).min(max_scroll as u16);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Get string input from user with proper echo and editing
pub fn get_input(prompt: &str, term: &mut Terminal) -> io::Result<String> {
    let mut input = String::new();

    loop {
        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(f.area());

            // Prompt
            let prompt_widget = Paragraph::new(prompt)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
                .style(Style::default().fg(Color::White));

            f.render_widget(prompt_widget, chunks[0]);

            // Input field
            let input_widget = Paragraph::new(input.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(input_widget, chunks[1]);

            // Help
            let help = Paragraph::new("Enter to submit | Backspace to delete")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            f.render_widget(help, chunks[2]);
        })?;

        // Handle input
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if !input.is_empty() {
                    break;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                input.pop();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            }) => {
                input.push(c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                input.clear();
                break;
            }
            _ => {}
        }
    }

    Ok(input)
}

/// Display menu with arrow key navigation
pub fn display_menu(title: &str, options: &[String], term: &mut Terminal) -> io::Result<usize> {
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
                .split(f.area());

            // Title
            let title_widget = Paragraph::new(title)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);

            f.render_widget(title_widget, chunks[0]);

            // Options list
            let items: Vec<ListItem> = options
                .iter()
                .map(|opt| ListItem::new(opt.as_str()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            f.render_stateful_widget(list, chunks[1], &mut list_state);

            // Help text
            let help = Paragraph::new("↑↓ to navigate | Enter to select | q to quit")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            f.render_widget(help, chunks[2]);
        })?;

        // Handle input
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let i = match list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            options.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                list_state.select(Some(i));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let i = match list_state.selected() {
                    Some(i) => {
                        if i >= options.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                list_state.select(Some(i));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                return Ok(list_state.selected().unwrap_or(0));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                return Ok(list_state.selected().unwrap_or(0));
            }
            _ => {}
        }
    }
}

/// Display decision menu with preview panel
pub fn display_decision_menu(
    title: &str,
    context: &str,
    choices: &[(String, String, String)],
    term: &mut Terminal,
) -> io::Result<usize> {
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut context_scroll: u16 = 0;

    loop {
        let selected = list_state.selected().unwrap_or(0);
        let size = term.terminal.size()?;
        
        // Calculate max scroll for context
        let context_lines = context.lines().count() + 2; // +2 for title
        let context_height = (size.height / 3).max(8) as usize; // Use top third, min 8 lines
        let max_context_scroll = context_lines.saturating_sub(context_height - 2) as u16;

        term.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length((size.height / 3).max(8)),  // Context - dynamic, larger
                    Constraint::Min(10),                            // Main content
                    Constraint::Length(3),                          // Help
                ])
                .split(f.area());

            // Title and context with scroll support
            let title_text = format!("━━━ {} ━━━\n\n{}", title, context);
            let title_widget = Paragraph::new(title_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(if max_context_scroll > 0 { "↑↓ to scroll context" } else { "" }))
                .wrap(Wrap { trim: true })
                .scroll((context_scroll, 0));

            f.render_widget(title_widget, main_chunks[0]);

            // Split middle section into choices and preview
            let middle_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(main_chunks[1]);

            // Choices list
            let items: Vec<ListItem> = choices
                .iter()
                .enumerate()
                .map(|(i, (label, _, _))| {
                    ListItem::new(format!("[{}] {}", i + 1, label))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("YOUR OPTIONS")
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            f.render_stateful_widget(list, middle_chunks[0], &mut list_state);

            // Preview panel
            let (_label, description, preview) = &choices[selected];
            let preview_text = format!("{}\n\n{}", description, preview);

            let preview_widget = Paragraph::new(preview_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("═══ WHAT YOU KNOW ═══")
                        .border_style(Style::default().fg(Color::Green)),
                )
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::White));

            f.render_widget(preview_widget, middle_chunks[1]);

            // Help text
            let help_lines = vec![
                Line::from("Tab/Shift+Tab: switch focus | ↑↓: navigate/scroll | Enter: decide | q: quit"),
                Line::from("(Real consequences unknown until after you commit)").style(Style::default().fg(Color::Red)),
            ];

            let help = Paragraph::new(help_lines)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(help, main_chunks[2]);
        })?;

        // Handle input with context scrolling
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                modifiers,
                ..
            }) => {
                if modifiers.contains(event::KeyModifiers::SHIFT) || max_context_scroll == 0 {
                    // Scroll choices list
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                choices.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                } else {
                    // Scroll context up
                    context_scroll = context_scroll.saturating_sub(1);
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                modifiers,
                ..
            }) => {
                if modifiers.contains(event::KeyModifiers::SHIFT) || max_context_scroll == 0 {
                    // Scroll choices list
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i >= choices.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                } else {
                    // Scroll context down
                    if context_scroll < max_context_scroll {
                        context_scroll += 1;
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                kind: KeyEventKind::Press,
                ..
            }) => {
                // Tab switches focus between context and choices
                let i = match list_state.selected() {
                    Some(i) => {
                        if i >= choices.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                list_state.select(Some(i));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                return Ok(selected);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                return Ok(selected);
            }
            _ => {}
        }
    }
}

/// Show decision outcome with formatted panels
pub fn show_decision_outcome(
    choice_label: &str,
    impact: &DecisionImpact,
    term: &mut Terminal,
) -> io::Result<()> {
    // Helper to extract risk changes
    let get_risk = |v: RiskVector| {
        impact
            .risk_delta
            .changes
            .get(&v)
            .map(|c| c.level_delta)
            .unwrap_or(0.0)
    };

    let outcome_text = format!(
        "You chose: {}\n\n\
         ═══ SECURITY IMPACT ═══\n\
         Data Exposure:    {:+.0}\n\
         Access Control:   {:+.0}\n\
         Detection:        {:+.0}\n\
         Vendor Risk:      {:+.0}\n\
         Insider Threat:   {:+.0}\n\n\
         ═══ BUSINESS IMPACT ═══\n\
         ARR Change:       ${:+.1}M\n\
         Velocity Change:  {:+.0}%\n\
         Churn Change:     {:+.1}%\n\
         Board Confidence: {:+.0}%\n\n\
         ═══ AUDIT TRAIL ═══\n\
         {}",
        choice_label,
        get_risk(RiskVector::DataExposure),
        get_risk(RiskVector::AccessControl),
        get_risk(RiskVector::Detection),
        get_risk(RiskVector::VendorRisk),
        get_risk(RiskVector::InsiderThreat),
        impact.business_delta.arr_change,
        impact.business_delta.velocity_change,
        impact.business_delta.churn_change,
        impact.business_delta.confidence_change,
        match impact.audit_trail {
            crate::core::types::AuditTrail::Clean => "✓ CLEAN - Defensible under scrutiny",
            crate::core::types::AuditTrail::Flagged => "⚠ FLAGGED - Questionable but not fatal",
            crate::core::types::AuditTrail::Toxic => "✗ TOXIC - Will be used against you in court",
        }
    );

    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(f.area());

        let outcome_widget = Paragraph::new(outcome_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("═══════════ DECISION OUTCOME ═══════════")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(outcome_widget, chunks[0]);

        let help = Paragraph::new("Press Enter to see alternate outcomes...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help, chunks[1]);
    })?;

    wait_for_enter()?;
    Ok(())
}

/// Show alternate outcomes
pub fn show_alternate_outcomes_with_impacts(
    chosen_idx: usize,
    choices: &[Choice],
    term: &mut Terminal,
) -> io::Result<()> {
    let mut text_lines = vec![
        format!("You chose: {}\n", choices[chosen_idx].label),
        String::from(""),
    ];

    for (idx, choice) in choices.iter().enumerate() {
        if idx != chosen_idx {
            text_lines.push(format!("═══ If you had chosen: {} ═══", choice.label));
            text_lines.push(String::from(""));
            text_lines.push(choice.description.clone());
            text_lines.push(String::from(""));
            text_lines.push(String::from("What you knew:"));

            if choice.impact_preview.estimated_arr_change != 0.0 {
                text_lines.push(format!(
                    "  Estimated ARR: ${:+.1}M",
                    choice.impact_preview.estimated_arr_change
                ));
            }
            if choice.impact_preview.budget_cost != 0.0 {
                text_lines.push(format!(
                    "  Budget Cost: ${:.2}M",
                    choice.impact_preview.budget_cost
                ));
            }
            if let Some(weeks) = choice.impact_preview.timeline_weeks {
                text_lines.push(format!("  Timeline: {} weeks", weeks));
            }
            if let Some(ref note) = choice.impact_preview.political_note {
                text_lines.push(format!("  Political: {}", note));
            }

            text_lines.push(String::from(""));
        }
    }

    let alternate_text = text_lines.join("\n");

    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(f.area());

        let widget = Paragraph::new(alternate_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("═══════════ WHAT IF YOU CHOSE DIFFERENTLY? ═══════════")
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        f.render_widget(widget, chunks[0]);

        let help = Paragraph::new("Press Enter to continue with your choice...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help, chunks[1]);
    })?;

    wait_for_enter()?;
    Ok(())
}

/// Display a status box with game information
pub fn display_box(title: &str, content: &str, term: &mut Terminal) -> io::Result<()> {
    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(f.area());

        let widget = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(widget, chunks[0]);

        let help = Paragraph::new("Press Enter to continue...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help, chunks[1]);
    })?;

    wait_for_enter()?;
    Ok(())
}

/// Display chapter/turn header
pub fn display_chapter_header(
    turn: u32,
    quarter: u32,
    phase: &str,
    term: &mut Terminal,
) -> io::Result<()> {
    let header_text = format!("TURN {} │ Q{} │ {}", turn, quarter, phase);

    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(f.area());

        let header = Paragraph::new(header_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        f.render_widget(header, chunks[0]);

        let help = Paragraph::new("Press Enter to continue...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help, chunks[1]);
    })?;

    wait_for_enter()?;
    Ok(())
}

/// Clear screen by redrawing empty frame
pub fn clear_screen(term: &mut Terminal) -> io::Result<()> {
    term.clear()
}

/// Print colored text (deprecated - use ratatui rendering instead)
pub fn print_colored(_text: &str, _color: crossterm::style::Color) -> io::Result<()> {
    // No-op for compatibility - use ratatui rendering in new code
    Ok(())
}

/// Typewriter effect (deprecated in ratatui context)
pub fn typewriter_effect(_text: &str, _delay_ms: u64) -> io::Result<()> {
    // No-op for compatibility
    Ok(())
}