use crate::core::state::{GameState, Ending, EventType, GamePhase};
use crate::core::types::{RiskVector, IncidentSeverity};
use colored::*;

pub fn display_ending(state: &GameState) {
    if let GamePhase::Ended(ending) = &state.phase {
        match ending {
            Ending::GoldenCSO => display_golden_cso(state),
            Ending::LawsuitSurvivor => display_lawsuit_survivor(state),
            Ending::PostBreachCleanup => display_post_breach_cleanup(state),
            Ending::CriminalInvestigation => display_criminal_investigation(state),
        }
    }
}

fn display_golden_cso(state: &GameState) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    println!("{}", "                    ENDING: GOLDEN CSO                     ".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════\n".bright_cyan());

    println!("{}", "LinkedIn Post - Sarah Chen, CEO".white().bold());
    println!("{}", "Posted 2 days ago · 12,847 views".bright_black());
    println!();
    
    println!("I'm proud to announce that {} has been promoted to VP of Security,", state.player.name.bright_cyan());
    println!("reporting directly to me. Over the past 16 months, they've transformed");
    println!("our security posture while enabling our fastest growth period ever.");
    println!();
    println!("Key achievements:");
    println!("  • Zero material breaches during hyper-growth phase");
    println!("  • SOC 2 Type II certification achieved 6 weeks early");
    println!("  • Security became a competitive advantage in enterprise sales");
    println!("  • ${:.1}M ARR growth without security-related friction", state.business.arr_millions - 12.0);
    println!();
    println!("What sets {} apart: they understand security as a business enabler,", state.player.name.bright_cyan());
    println!("not a blocker. Every decision was transparent, every risk documented,");
    println!("every trade-off justified. That's the kind of leadership we need.");
    println!();
    println!("{}", "1,247 reactions · 89 comments".bright_black());
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    println!();
    println!("{}", "Three weeks later...".white().italic());
    println!();
    println!("Subject: Opportunity at Fortune 500 Company");
    println!("From: Executive Recruiter");
    println!();
    println!("We're conducting a confidential search for a VP of Security role");
    println!("at a Fortune 500 financial services company. Your reputation for");
    println!("balancing security and business growth has come highly recommended.");
    println!();
    println!("Compensation: $450K base + equity + bonus");
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    println!();
    display_final_stats(state);
    println!();
    println!("{}", "Achievement Unlocked: Golden CSO (Top 5%)".bright_yellow().bold());
    println!("{}", "You survived with credibility intact.".white());
}

fn display_lawsuit_survivor(state: &GameState) {
    let fine = 5.0 * state.narrative.get_multiplier();

    println!("\n{}", "═══════════════════════════════════════════════════════════".yellow());
    println!("{}", "                ENDING: LAWSUIT SURVIVOR                   ".yellow().bold());
    println!("{}", "═══════════════════════════════════════════════════════════\n".yellow());

    println!("{}", "SEC Filing - Form 8-K".white().bold());
    println!("{}", "Item 8.01 - Material Events".bright_black());
    println!();
    
    println!("On [DATE], the Company entered into a settlement agreement with");
    println!("the Federal Trade Commission regarding a data security incident");
    println!("that occurred in Q3 2025.");
    println!();
    println!("Settlement Terms:");
    println!("  • Civil penalty: ${:.1} million", fine);
    println!("  • Consent decree: 20-year privacy monitoring program");
    println!("  • Independent security assessments: biannual for 5 years");
    println!();
    println!("The Company has implemented enhanced security controls and");
    println!("restructured its information security program under continued");
    println!("leadership of its current Chief Security Officer.");
    println!();
    println!("{}", "Note: settlement includes no admission of wrongdoing.".bright_black().italic());
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".yellow());
    println!();
    println!("{}", "Email - From: CEO".white().bold());
    println!("{}", "Subject: Your Performance Improvement Plan".bright_black());
    println!();
    println!("We need to discuss your objectives for the next 90 days.");
    println!();
    println!("The board has expressed concern about some of the decisions made");
    println!("during the incident. While we're not making changes to your role,");
    println!("we are bringing in an external consultant to 'assist' with the");
    println!("remediation program.");
    println!();
    println!("Let's schedule time tomorrow.");
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".yellow());
    println!();
    display_final_stats(state);
    println!();
    println!("{}", "Achievement: Lawsuit Survivor (Middle 70%)".yellow().bold());
    println!("{}", "You kept your job. Barely.".white());
}

fn display_post_breach_cleanup(state: &GameState) {
    let fine = 20.0 * state.narrative.get_multiplier();
    let total_impacted: u32 = state.active_incidents.iter().filter_map(|i| i.customer_impact_count).sum();
    let impacted = if total_impacted > 0 { total_impacted } else { 840000 };

    println!("\n{}", "═══════════════════════════════════════════════════════════".red());
    println!("{}", "             ENDING: POST-BREACH CLEANUP CREW              ".red().bold());
    println!("{}", "═══════════════════════════════════════════════════════════\n".red());

    println!("{}", "Bloomberg News".white().bold());
    println!("{}", "Breaking News · 47 minutes ago".bright_black());
    println!();
    
    println!("{}", format!("[COMPANY] Data Breach Exposes {} Customer Records", impacted).red().bold());
    println!();
    println!("Regulators impose ${:.0}M fine after security chief testified that", fine);
    println!("warning signs were ignored for months. Internal documents show");
    println!("CSO minimized breach scope in initial disclosure.");
    println!();
    println!("Class-action lawsuit names former CSO {} personally,", state.player.name.red());
    println!("alleging gross negligence and breach of fiduciary duty.");
    println!();
    println!("CEO Sarah Chen terminated 'by mutual agreement' yesterday.");
    println!("Three board members resigned this morning.");
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".red());
    println!();
    println!("{}", "DISCOVERY PHASE: YOUR DECISIONS ON TRIAL".red().bold());
    println!();
    
    // Show the player's contradictions
    if state.narrative.score < 50.0 {
        println!("{}", "Evidence presented in legal proceedings:".white().bold());
        println!();
        
        println!("  {} Your Q2 Board Report:", "▸".red());
        println!("    'Cloud migration on track, no material risks'");
        println!();
        println!("  {} Your Slack to Engineering Lead (same week):", "▸".red());
        println!("    'I'm worried about IAM config but we need to ship'");
        println!();
        println!("  {} Forensics Timeline:", "▸".red());
        println!("    S3 bucket public: {} days before breach discovery", 47);
        println!("    Your system access logs: viewed misconfiguration warning 3 times");
        println!();
        println!("  {} Regulator's Conclusion:", "▸".red().bold());
        println!("    'Pattern of concealment. Bad faith assumed.'");
        println!("    Penalty Multiplier: ×{:.1}", state.narrative.get_multiplier());
        println!();
    }
    
    println!("{}", "═══════════════════════════════════════════════════════════".red());
    println!();
    println!("{}", "Six months later...".white().italic());
    println!();
    println!("Your LinkedIn profile now reads:");
    println!("  'Fractional CISO | Security Consultant'");
    println!();
    println!("No Fortune 500 company will hire you full-time.");
    println!("Your name is the first result when recruiters Google 'security breach CSO'.");
    println!();
    println!("Your deposition transcripts are public record.");
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".red());
    println!();
    
    // Replay key decisions
    println!("{}", "WHAT YOU COULD HAVE DONE DIFFERENTLY:".red().bold());
    println!();
    replay_critical_decisions(state);
    println!();
    
    display_final_stats(state);
    println!();
    println!("{}", "Achievement: Post-Breach Cleanup Crew (Bottom 25%)".red().bold());
    println!("{}", "Your story didn't survive discovery.".white());
}

fn display_criminal_investigation(state: &GameState) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".bright_red());
    println!("{}", "             ENDING: CRIMINAL INVESTIGATION               ".bright_red().bold());
    println!("{}", "═══════════════════════════════════════════════════════════\n".bright_red());

    println!("{}", "U.S. Department of Justice Press Release".white().bold());
    println!("{}", "Former Tech Executive Indicted in Data Breach Cover-Up".bright_black());
    println!();
    
    println!("A federal grand jury returned an indictment against former CSO {}", state.player.name);
    println!("charging them with securities fraud, wire fraud, and obstruction of justice.");
    println!();
    println!("The indictment alleges that the defendant:");
    println!("  • Concealed a series of security breaches affecting millions of users");
    println!("  • Falsified compliance reports to the board and regulators");
    println!("  • Directed the deletion of audit logs during an active investigation");
    println!();
    println!("If convicted, the defendant faces up to 20 years in federal prison.");
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".bright_red());
    println!();
    println!("{}", "Email - From: Your Attorney".white().bold());
    println!("{}", "Subject: Urgent: Federal Indictment".bright_black());
    println!();
    println!("We need to meet immediately. Do not speak to anyone.");
    println!("Secure all devices and communications.");
    println!();
    println!("The charges are serious, but we have options.");
    println!("Plea negotiations start next week.");
    println!();
    
    if state.narrative.criminal_exposure() {
        println!("{}", "Key Evidence from Discovery:".red().bold());
        println!();
        for buried in &state.narrative.buried_incidents {
            println!("  • Buried Incident: {} (Reported as {:?}, Actual {:?})", buried.incident_id, buried.reported_severity, buried.actual_severity);
        }
        for delayed in &state.narrative.delayed_escalations {
            println!("  • Delayed Escalation: {} (Should have: Turn {}, Actual: Turn {})", delayed.incident_id, delayed.should_have_escalated_turn, delayed.actually_escalated_turn);
        }
    }
    
    println!();
    println!("{}", "═══════════════════════════════════════════════════════════".bright_red());
    println!();
    display_final_stats(state);
    println!();
    println!("{}", "Achievement: Criminal Investigation (Bottom 1%)".bright_red().bold());
    println!("{}", "Lawyer up. Your decisions led to personal liability.".white());
}

fn display_final_stats(state: &GameState) {
    println!("{}", "═══════════════════════════════════════════════════════════".white());
    println!("{}", "                      FINAL METRICS                        ".white().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".white());
    println!();
    
    println!("{}", "Business Impact:".cyan().bold());
    println!("  ARR:                    ${:.1}M (started at $12.0M)", state.business.arr_millions);
    println!("  Roadmap Velocity:       {:.0}%", state.business.roadmap_velocity_percent);
    println!("  Customer Churn Risk:    {:.1}%", state.business.customer_churn_probability);
    println!("  Board Confidence:       {:.0}%", state.business.board_confidence_percent);
    println!();
    
    println!("{}", "Risk Exposure:".yellow().bold());
    let get_level = |v: RiskVector| state.risk.vectors.get(&v).map_or(0.0, |m| m.current_level);
    println!("  Data Exposure:          {:.0}%", get_level(RiskVector::DataExposure));
    println!("  Access Control:         {:.0}%", get_level(RiskVector::AccessControl));
    println!("  Detection Gaps:         {:.0}%", get_level(RiskVector::Detection));
    println!("  Vendor Risk:            {:.0}%", get_level(RiskVector::VendorRisk));
    println!("  Insider Threat:         {:.0}%", get_level(RiskVector::InsiderThreat));
    println!();
    
    println!("{}", "Narrative Integrity:".magenta().bold());
    println!("  Credibility Score:      {:.0}%", state.narrative.score);
    println!("  Inconsistencies:        {}", state.narrative.inconsistencies.len());
    println!("  Buried Incidents:       {}", state.narrative.buried_incidents.len());
    println!("  Delayed Escalations:    {}", state.narrative.delayed_escalations.len());
    println!("  Penalty Multiplier:     ×{:.1}", state.narrative.get_multiplier());
    println!();
    
    println!("{}", "Material Incidents:".red().bold());
    let critical = state.active_incidents.iter().filter(|i| i.severity == IncidentSeverity::Critical).count();
    let high = state.active_incidents.iter().filter(|i| i.severity == IncidentSeverity::High).count();
    println!("  Critical:               {}", critical);
    println!("  High:                   {}", high);
    println!();
    
    println!("{}", "Budget Management:".green().bold());
    println!("  Total Annual Budget:    ${:.1}M", state.budget.total_annual);
    println!("  Spent:                  ${:.1}M", state.budget.spent);
    println!("  Remaining:              ${:.1}M", state.budget.available());
    println!();
}

fn replay_critical_decisions(state: &GameState) {
    // Find decisions that led to narrative integrity loss
    for event in state.events.iter().filter(|e| matches!(e.event_type, EventType::DecisionMade)) {
        if let Some(decision_id) = &event.decision_id {
            if decision_id.contains("minimize") || decision_id.contains("accept_risk") || decision_id.contains("defer") {
                println!("  {} Turn {}: {}", "▸".red(), event.turn, event.description);
                println!("    Alternative: [Consider proactive disclosure or risk mitigation]");
                println!();
            }
        }
    }
}

pub fn display_turn_header(turn: u32, quarter: u32, phase: &GamePhase) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".bright_blue());
    println!("{} {} | Q{} | Phase: {:?}", "TURN".bright_blue().bold(), turn, quarter, phase);
    println!("{}", "═══════════════════════════════════════════════════════════\n".bright_blue());
}

pub fn display_status(state: &GameState) {
    println!("{}", "Current Status:".white().bold());
    println!("  ARR: ${:.1}M | Board Confidence: {:.0}% | Integrity: {:.0}%", 
             state.business.arr_millions,
             state.business.board_confidence_percent,
             state.narrative.score);
    println!("  Risk Total: {:.0} | Budget Available: ${:.2}M\n",
             state.risk.total_exposure,
             state.budget.available());
}