use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use textwrap::wrap;

// Import types needed for the UI logic
use crate::core::types::{RiskVector, DecisionImpact};

#[derive(Debug)]
pub struct Terminal {
    width: u16,
    height: u16,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self { width, height })
    }

    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
}

/// Enum to represent wait results for handling resizes during pauses
#[derive(Debug, PartialEq)]
enum WaitResult {
    Continue,
    Resized,
}

/// Display paginated text that user can advance with Enter.
/// Handles terminal resizes by redrawing the current page.
pub fn display_paginated_text(text: &str, term: &mut Terminal) -> io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut page = 0;
    let mut total_pages;

    loop {
        clear_screen()?;

        let wrapped = wrap(text, term.width() - 4);
        let lines: Vec<&str> = wrapped.iter().map(|s| s.as_ref()).collect();

        let lines_per_page = term.height() - 3; // Reserve space for prompt
        total_pages = (lines.len() + lines_per_page as usize - 1) / lines_per_page as usize;

        if page >= total_pages {
            break;
        }

        let start = page * lines_per_page as usize;
        let end = std::cmp::min(start + lines_per_page as usize, lines.len());

        for line in &lines[start..end] {
            println!("{}", line);
        }

        println!();
        if page < total_pages - 1 {
            print_colored("Press Enter to continue...", Color::DarkGrey)?;
            io::stdout().flush()?;

            if wait_for_enter(term)? == WaitResult::Resized {
                continue; // Redraw current page on resize
            }
        } else {
            break;
        }

        page += 1;
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

/// Get string input from user with event-based handling for better control.
/// Supports backspace and handles resizes without interrupting input.
pub fn get_input(prompt: &str, term: &mut Terminal) -> io::Result<String> {
    terminal::enable_raw_mode()?;

    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    let mut cursor_pos = 0;

    loop {
        match event::read()? {
            Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Enter => {
                        println!();
                        break;
                    }
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            input.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                            execute!(
                                io::stdout(),
                                cursor::MoveLeft(1),
                                terminal::Clear(ClearType::FromCursorDown)
                            )?;
                            print!("{}", &input[cursor_pos..]);
                            execute!(io::stdout(), cursor::MoveLeft((input.len() - cursor_pos) as u16))?;
                        }
                    }
                    KeyCode::Char(c) => {
                        input.insert(cursor_pos, c);
                        cursor_pos += 1;
                        print!("{}", &input[cursor_pos - 1..]);
                        execute!(io::stdout(), cursor::MoveLeft((input.len() - cursor_pos) as u16))?;
                        io::stdout().flush()?;
                    }
                    KeyCode::Left => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                        }
                    }
                    KeyCode::Right => {
                        if cursor_pos < input.len() {
                            cursor_pos += 1;
                            execute!(io::stdout(), cursor::MoveRight(1))?;
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(w, h) => {
                term.update_size(w, h);
                // Redraw prompt and current input on resize
                clear_screen()?;
                print!("{}{}", prompt, input);
                execute!(io::stdout(), cursor::MoveLeft((input.len() - cursor_pos) as u16))?;
                io::stdout().flush()?;
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    Ok(input.trim().to_string())
}

/// Display menu with arrow key navigation.
/// Handles resizes by redrawing immediately.
pub fn display_menu(title: &str, options: &[String], term: &mut Terminal) -> io::Result<usize> {
    terminal::enable_raw_mode()?;

    let mut selected = 0;

    loop {
        clear_screen()?;

        // Display title
        println!("{}", title);
        println!();

        // Display options
        for (idx, option) in options.iter().enumerate() {
            if idx == selected {
                print_colored(&format!("▶ {}", option), Color::Cyan)?;
            } else {
                println!("  {}", option);
            }
        }

        println!();
        print_colored("Use ↑↓ arrows to navigate, Enter to select", Color::DarkGrey)?;
        io::stdout().flush()?;

        // Handle input
        match event::read()? {
            Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < options.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
            Event::Resize(w, h) => {
                term.update_size(w, h);
                // Loop will redraw
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    Ok(selected)
}

/// Display decision with minimal preview (only business info).
/// Handles resizes by redrawing the entire menu.
pub fn display_decision_menu(
    title: &str,
    context: &str,
    choices: &[(String, String, String)], // (label, description, simple_preview)
    term: &mut Terminal,
) -> io::Result<usize> {
    terminal::enable_raw_mode()?;

    let mut selected = 0;

    loop {
        clear_screen()?;

        // Display title
        print_colored(&format!("━━━ {} ━━━", title), Color::Cyan)?;
        println!("\n");

        // Display context (paginated if needed)
        let wrapped_context = wrap(context, term.width() - 4);
        for line in wrapped_context.iter().take(10) {
            println!("{}", line);
        }

        println!();
        print_colored("YOUR OPTIONS:", Color::Yellow)?;
        println!();

        // Display choices with selection indicator
        for (idx, (label, _desc, _preview)) in choices.iter().enumerate() {
            if idx == selected {
                print_colored(&format!("▶ [{}] {}", idx + 1, label), Color::Cyan)?;
            } else {
                println!("  [{}] {}", idx + 1, label);
            }
        }

        println!();

        // Display minimal info for selected choice
        let (_label, description, preview) = &choices[selected];
        print_colored("═══ WHAT YOU KNOW ═══", Color::Green)?;
        println!("\n{}", description);
        println!("\n{}", preview);

        println!();
        print_colored("Use ↑↓ to navigate, Enter to decide", Color::DarkGrey)?;
        print_colored("(Real consequences unknown until after you commit)", Color::DarkGrey)?;
        io::stdout().flush()?;

        // Handle input
        match event::read()? {
            Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < choices.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
            Event::Resize(w, h) => {
                term.update_size(w, h);
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    Ok(selected)
}

/// Show the actual outcome after decision is made.
pub fn show_decision_outcome(
    choice_label: &str,
    impact: &DecisionImpact,
    term: &mut Terminal,
) -> io::Result<()> {
    terminal::enable_raw_mode()?;

    clear_screen()?;

    print_colored("═══════════════════════════════════════════", Color::Cyan)?;
    print_colored("           DECISION OUTCOME", Color::Cyan)?;
    print_colored("═══════════════════════════════════════════\n", Color::Cyan)?;

    println!("You chose: {}\n", choice_label);

    print_colored("═══ SECURITY IMPACT ═══", Color::Yellow)?;
    
    // FIXED: Helper to safely extract risk changes from the HashMap
    // We map to level_delta because RiskChange is a struct, not an f64.
    let get_risk = |v: RiskVector| {
        impact.risk_delta.changes.get(&v)
            .map(|c| c.level_delta)
            .unwrap_or(0.0)
    };

    println!("Data Exposure:    {:+.0}", get_risk(RiskVector::DataExposure));
    println!("Access Control:   {:+.0}", get_risk(RiskVector::AccessControl));
    println!("Detection:        {:+.0}", get_risk(RiskVector::Detection));
    println!("Vendor Risk:      {:+.0}", get_risk(RiskVector::VendorRisk));
    println!("Insider Threat:   {:+.0}", get_risk(RiskVector::InsiderThreat));

    println!();
    print_colored("═══ BUSINESS IMPACT ═══", Color::Green)?;
    println!("ARR Change:       ${:+.1}M", impact.business_delta.arr_change);
    println!("Velocity Change:  {:+.0}%", impact.business_delta.velocity_change);
    println!("Churn Change:     {:+.1}%", impact.business_delta.churn_change);
    println!("Board Confidence: {:+.0}%", impact.business_delta.confidence_change);

    println!();
    print_colored("═══ AUDIT TRAIL ═══", Color::Magenta)?;
    let trail_text = match impact.audit_trail {
        crate::core::types::AuditTrail::Clean => "✓ CLEAN - Defensible under scrutiny",
        crate::core::types::AuditTrail::Flagged => "⚠ FLAGGED - Questionable but not fatal",
        crate::core::types::AuditTrail::Toxic => "✗ TOXIC - Will be used against you in court",
    };
    println!("{}", trail_text);

    println!();
    print_colored("Press Enter to see alternate outcomes...", Color::DarkGrey)?;
    io::stdout().flush()?;

    wait_for_enter(term)?;

    terminal::disable_raw_mode()?;
    Ok(())
}

/// Show alternate reality - what if you chose differently? (with full impacts)
pub fn show_alternate_outcomes_with_impacts(
    chosen_idx: usize,
    choices: &[crate::core::decisions::Choice],
    term: &mut Terminal,
) -> io::Result<()> {
    terminal::enable_raw_mode()?;

    clear_screen()?;

    print_colored("═══════════════════════════════════════════", Color::Magenta)?;
    print_colored("        WHAT IF YOU CHOSE DIFFERENTLY?", Color::Magenta)?;
    print_colored("═══════════════════════════════════════════\n", Color::Magenta)?;

    println!("You chose: {}\n", choices[chosen_idx].label);

    // Show what would have happened with other choices
    for (idx, choice) in choices.iter().enumerate() {
        if idx != chosen_idx {
            print_colored(&format!("═══ If you had chosen: {} ═══", choice.label), Color::Yellow)?;
            println!();

            // We need to calculate what the impact would have been
            // For now, show the description
            println!("{}", choice.description);

            // Show business preview they saw
            println!();
            println!("What you knew:");
            if choice.impact_preview.estimated_arr_change != 0.0 {
                println!("  Estimated ARR: ${:+.1}M", choice.impact_preview.estimated_arr_change);
            }
            if choice.impact_preview.budget_cost != 0.0 {
                println!("  Budget Cost: ${:.2}M", choice.impact_preview.budget_cost);
            }
            if let Some(weeks) = choice.impact_preview.timeline_weeks {
                println!("  Timeline: {} weeks", weeks);
            }
            if let Some(ref note) = choice.impact_preview.political_note {
                println!("  Political: {}", note);
            }

            println!();
        }
    }

    print_colored("Press Enter to continue with your choice...", Color::DarkGrey)?;
    io::stdout().flush()?;

    wait_for_enter(term)?;

    terminal::disable_raw_mode()?;
    Ok(())
}

/// Print colored text
pub fn print_colored(text: &str, color: Color) -> io::Result<()> {
    execute!(
        io::stdout(),
        SetForegroundColor(color),
        Print(text),
        ResetColor,
        Print("\n")
    )
}

/// Clear the screen
pub fn clear_screen() -> io::Result<()> {
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
}

/// Wait for Enter key, returning if resized or continued.
fn wait_for_enter(term: &mut Terminal) -> io::Result<WaitResult> {
    loop {
        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => return Ok(WaitResult::Continue),
            Event::Resize(w, h) => {
                term.update_size(w, h);
                return Ok(WaitResult::Resized);
            }
            _ => {}
        }
    }
}

/// Display a box with text
pub fn display_box(title: &str, content: &str, term: &Terminal) -> io::Result<()> {
    let width = term.width().min(80) as usize;
    let border = "═".repeat(width);

    println!("╔{}╗", border);
    println!("║ {:<width$} ║", title, width = width - 2);
    println!("╠{}╣", border);

    let wrapped = wrap(content, width - 4);
    for line in wrapped {
        println!("║ {:<width$} ║", line, width = width - 2);
    }

    println!("╚{}╝", border);

    Ok(())
}

/// Typewriter effect for dramatic moments
pub fn typewriter_effect(text: &str, delay_ms: u64) -> io::Result<()> {
    for ch in text.chars() {
        print!("{}", ch);
        io::stdout().flush()?;
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    }
    println!();
    Ok(())
}

/// Display chapter/turn header
pub fn display_chapter_header(turn: u32, quarter: u32, phase: &str, term: &Terminal) -> io::Result<()> {
    clear_screen()?;

    let width = term.width().min(80) as usize;
    let border = "═".repeat(width);

    println!("\n╔{}╗", border);
    println!("║{:^width$}║", format!("TURN {} │ Q{} │ {}", turn, quarter, phase), width = width);
    println!("╚{}╝\n", border);

    Ok(())
}