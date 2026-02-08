// main.rs
use cso_simulator::core::{DecisionFactory, DecisionLoader, GameError, GamePhase, GameState, ImpactPreview, Player, Result};
use cso_simulator::narrative::{display_ending};
use cso_simulator::ui::*;
use cso_simulator::GamePersistence;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::style::Color;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize terminal
    let mut term = Terminal::new().map_err(|_| GameError::SystemFailure)?;

    // Display intro
    display_intro(&mut term)?;

    // Get player name and company
    let player = create_player(&mut term)?;

    // Initialize game state
    let mut state = GameState::new(player.clone());
    let save_path = PathBuf::from("./cso_save.enc");

    // Try to load decision config
    let decision_loader = DecisionLoader::new().unwrap_or_else(|_| DecisionLoader { decisions: Default::default() });

    // Main game loop
    loop {
        // Check if game is over
        if matches!(state.phase, GamePhase::Ended(_)) {
            display_ending(&state);
            break;
        }

        // Display turn information
        let phase_name = match &state.phase {
            GamePhase::InheritanceDisaster => "Inheritance Disaster",
            GamePhase::OperationalTempo => "Operational Tempo",
            GamePhase::Discovery => "Discovery",
            GamePhase::Ended(_) => "Ended",
        };

        display_chapter_header(state.turn, state.quarter, phase_name, &term)?;
        display_status(&state, &term)?;
        wait_for_enter()?;

        // Check for risk materialization
        let materialized = state.materialize_risks();
        if !materialized.is_empty() {
            clear_screen()?;
            print_colored("⚠ RISK MATERIALIZED ⚠", Color::Red)?;
            println!();
            for incident in &materialized {
                display_paginated_text(incident, &mut term)?;
            }
            wait_for_enter()?;
        }

        // Get decision for this turn
        if let Some(mut decision) = decision_loader.get_decision(state.turn).cloned()
            .or_else(|| DecisionFactory::generate_decision(&state))
        {
            // Prepare choices for UI - only show business info
            let choice_data: Vec<(String, String, String)> = decision.choices.iter()
                .map(|c| (
                    c.label.clone(),
                    c.description.clone(),
                    format_simple_preview(&c.impact_preview)
                ))
                .collect();

            // Display decision and get choice
            let chosen_idx = display_decision_menu(
                &decision.title,
                &decision.context,
                &choice_data,
                &mut term,
            )?;

            let choice_id = decision.choices[chosen_idx].id.clone();
            let choice_label = decision.choices[chosen_idx].label.clone();

            // Apply the choice
            let impact = decision.apply_choice(&choice_id, &mut state)?;

            // NOW show the full outcome
            show_decision_outcome(&choice_label, &impact, &mut term)?;

            // Show alternate outcomes with what they would have gotten
            show_alternate_outcomes_with_impacts(chosen_idx, &decision.choices, &mut term)?;

            print_colored("✓ Decision recorded in audit log.", Color::DarkGrey)?;
            wait_for_enter()?;
        } else {
            clear_screen()?;
            display_paginated_text(
                "No major decisions this turn. Operations continue normally.\n\nYour team handles day-to-day security operations while you prepare for the next board meeting.",
                &mut term
            )?;
            wait_for_enter()?;
        }

        // Advance to next turn
        state.advance_turn();

        // Auto-save after each turn
        let persistence = GamePersistence::new("cso-game-2025")?;
        if let Err(_) = persistence.save(&state, &save_path) {
            print_colored("⚠ Failed to save game", Color::Yellow)?;
        }
    }

    Ok(())
}

fn display_intro(term: &mut Terminal) -> Result<()> {
    clear_screen()?;

    let intro_text = r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║           CSO JUDGMENT SIMULATOR v1.0                     ║
║           A Security Failure RPG                          ║
║                                                           ║
║   Tagline: Every decision is a liability.                ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝

A narrative simulation of how security decisions turn into legal outcomes.

You are about to become a Chief Security Officer.
The previous CSO 'left to pursue other opportunities.'

Risk doesn't fail fast—it accretes silently.
Bad decisions compound.
This game doesn't punish you immediately.
It audits you later.

Just like reality.
"#;

    display_paginated_text(intro_text, term)?;
    wait_for_enter()?;

    Ok(())
}

fn create_player(term: &mut Terminal) -> Result<Player> {
    clear_screen()?;

    print_colored("═══ CHARACTER CREATION ═══", Color::Cyan)?;
    println!();

    let name = get_input("Enter your name: ", term)
        .map_err(|_| GameError::SystemFailure)?;

    println!();

    // Generate company name options
    let companies = vec![
        "TechFlow Solutions".to_string(),
        "DataSync Inc.".to_string(),
        "CloudVault Systems".to_string(),
        "NexGen Analytics".to_string(),
        "SecureStack Technologies".to_string(),
    ];

    println!("Select your company:\n");
    let company_idx = display_menu("Choose your company:", &companies, term)?;
    let company_name = companies[company_idx].clone();

    clear_screen()?;
    print_colored(&format!("Welcome, {}!", name), Color::Green)?;
    print_colored(&format!("You are now the CSO of {}", company_name), Color::Green)?;
    println!();
    wait_for_enter()?;

    Ok(Player::new(name, company_name, "CSO".to_string()))
}

fn display_status(state: &GameState, term: &Terminal) -> Result<()> {
    display_box(
        "CURRENT STATUS",
        &format!(
            "CSO: {} | Company: {}\n\
             ARR: ${:.1}M | Board Confidence: {:.0}% | Integrity: {:.0}%\n\
             Risk Total: {:.0} | Budget Available: ${:.2}M",
            state.player.name,
            state.player.company_name,
            state.business.arr_millions,
            state.business.board_confidence_percent,
            state.narrative.score,
            state.risk.total_exposure,
            state.budget.available()
        ),
        term
    )?;

    println!();
    Ok(())
}

fn format_simple_preview(preview: &ImpactPreview) -> String {
    let mut lines = vec![];

    // Business info only - what you know before deciding
    if preview.estimated_arr_change != 0.0 {
        lines.push(format!("Estimated ARR Impact: ${:+.1}M", preview.estimated_arr_change));
    }

    if preview.budget_cost != 0.0 {
        lines.push(format!("Budget Cost: ${:.2}M", preview.budget_cost));
    }

    if let Some(weeks) = preview.timeline_weeks {
        lines.push(format!("Timeline: {} weeks", weeks));
    }

    if let Some(ref note) = preview.political_note {
        lines.push(format!("\nPolitical Context: {}", note));
    }

    if lines.is_empty() {
        lines.push("No immediate financial impact".to_string());
    }

    lines.join("\n")
}

/// Simple wait for enter without resize handling
pub fn wait_for_enter() -> Result<()> {
    loop {
        if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event::read()
            .map_err(|_| GameError::SystemFailure)? {
            return Ok(());
        }
    }
}