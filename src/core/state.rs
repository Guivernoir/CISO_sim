use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::core::types::*;
use std::collections::HashMap;

/// Immutable event in the audit log - everything is recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub turn: u32,
    pub event_type: EventType,
    pub description: String,
    pub decision_id: Option<String>,
    pub visibility: EventVisibility,  // Who knows about this?
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EventVisibility {
    Internal,      // Only security team
    Management,    // C-suite knows
    Board,         // Board was informed
    Public,        // Disclosed externally
    Buried,        // Someone tried to hide this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    GameStart,
    DecisionMade,
    RiskMaterialized,
    BoardPressure,
    ComplianceAudit,
    IncidentDetected,
    IncidentEscalated,
    IncidentResolved,
    QuarterEnd,
    BoardReview,
    TeamMemberDeparted,
    TeamMemberHired,
    ComplianceFindingOpened,
    ComplianceFindingClosed,
    PoliticalCapitalSpent,
    ReputationChange,
    GameEnd,
}

/// Core game state - now significantly more complex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub turn: u32,
    pub quarter: u32,
    pub risk: RiskLevel,
    pub business: BusinessMetrics,
    pub narrative: NarrativeIntegrity,
    pub budget: Budget,
    pub political_capital: PoliticalCapital,
    pub team: SecurityTeam,
    pub compliance: ComplianceStatus,
    pub threat_landscape: ThreatLandscape,
    pub board: Vec<BoardMember>,
    pub events: Vec<Event>,
    pub decisions_made: Vec<String>,
    pub active_incidents: Vec<ActiveIncident>,
    pub resolved_incidents: Vec<ResolvedIncident>,
    pub phase: GamePhase,
    pub quarterly_objectives: Vec<Objective>,
    pub technical_debt: TechnicalDebt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GamePhase {
    InheritanceDisaster,  // Turns 1-3: "What did I walk into?"
    OperationalTempo,     // Turns 4-12: "Just keep the lights on"
    Discovery,            // Turns 13-16: "The auditors found what?"
    Ended(Ending),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Ending {
    GoldenCISO,           // Top 5%: Nailed it
    LawsuitSurvivor,     // Middle 70%: You made it out alive
    PostBreachCleanup,   // Bottom 25%: Resume update time
    CriminalInvestigation, // Bottom 1%: Lawyer up
}

/// Active incidents - require response and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveIncident {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub turn_detected: u32,
    pub turn_deadline: Option<u32>,  // When does this blow up?
    pub escalated_to_board: bool,
    pub escalation_turn: Option<u32>,
    pub response_status: IncidentResponseStatus,
    pub assigned_team: Vec<String>,
    pub capacity_consumed: f64,
    pub containment_percent: f64,
    pub root_cause_identified: bool,
    pub public_disclosure_required: bool,
    pub customer_impact_count: Option<u32>,
    pub timeline: Vec<IncidentTimelineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentTimelineEntry {
    pub turn: u32,
    pub action: String,
    pub actor: String,
    pub visibility: EventVisibility,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IncidentResponseStatus {
    Detected,
    Investigating,
    Containing,
    Eradicating,
    Recovering,
    PostMortem,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedIncident {
    pub id: String,
    pub original_incident: String,
    pub resolution_turn: u32,
    pub time_to_resolve: u32,
    pub lessons_learned: Vec<String>,
    pub follow_up_actions: Vec<String>,
    pub final_cost: f64,
    pub reputation_impact: f64,
}

/// Objectives - what the board expects you to accomplish
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,
    pub description: String,
    pub assigned_quarter: u32,
    pub priority: ObjectivePriority,
    pub progress: f64,  // 0-100
    pub completion_turn: Option<u32>,
    pub assigned_by: BoardMemberRole,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ObjectivePriority {
    Critical,  // Failure = termination
    High,
    Medium,
    Low,
}

/// Technical debt - the gift that keeps on giving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub total_debt_points: f64,
    pub debt_velocity: f64,  // How fast debt is growing
    pub categories: HashMap<DebtCategory, f64>,
    pub oldest_debt_age_turns: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DebtCategory {
    UnpatchedSystems,
    LegacyAccess,
    UndocumentedProcesses,
    ToolingGaps,
    ArchitecturalFlaws,
    ComplianceGaps,
}

impl TechnicalDebt {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert(DebtCategory::UnpatchedSystems, 40.0);
        categories.insert(DebtCategory::LegacyAccess, 30.0);
        categories.insert(DebtCategory::UndocumentedProcesses, 25.0);
        categories.insert(DebtCategory::ToolingGaps, 35.0);
        categories.insert(DebtCategory::ArchitecturalFlaws, 20.0);
        categories.insert(DebtCategory::ComplianceGaps, 30.0);

        Self {
            total_debt_points: 180.0,  // You inherit a mess
            debt_velocity: 5.0,
            categories,
            oldest_debt_age_turns: 12,  // Some of this is ancient
        }
    }

    pub fn accumulate(&mut self, amount: f64, category: DebtCategory) {
        self.total_debt_points += amount;
        *self.categories.entry(category).or_insert(0.0) += amount;
    }

    pub fn pay_down(&mut self, amount: f64, category: DebtCategory) -> f64 {
        let current = self.categories.get(&category).copied().unwrap_or(0.0);
        let actual_reduction = amount.min(current);
        
        *self.categories.entry(category).or_insert(0.0) -= actual_reduction;
        self.total_debt_points -= actual_reduction;
        
        actual_reduction
    }

    /// Debt increases risk and slows everything down
    pub fn get_risk_multiplier(&self) -> f64 {
        1.0 + (self.total_debt_points / 200.0)
    }

    pub fn get_velocity_penalty(&self) -> f64 {
        (self.total_debt_points / 10.0).min(50.0)  // Max 50% penalty
    }
}

impl GameState {
    pub fn new(player: Player) -> Self {
        let mut events = Vec::new();
        events.push(Event {
            timestamp: Utc::now(),
            turn: 0,
            event_type: EventType::GameStart,
            description: format!(
                "{} appointed as CISO of {}. Previous CISO '{}' left to 'pursue other opportunities'. \
                Exit interview mentions: 'Board expectations unrealistic', 'Budget insufficient', \
                'Nobody listened until after the breach'.",
                player.name, player.company_name,
                if rand::random::<bool>() { "Richard" } else { "Susan" }
            ),
            decision_id: None,
            visibility: EventVisibility::Management,
            metadata: HashMap::new(),
        });

        // Initialize board with personalities
        let board = Self::initialize_board();

        // Set initial quarterly objectives
        let quarterly_objectives = Self::initial_objectives(&board);

        Self {
            player,
            turn: 1,
            quarter: 1,
            risk: RiskLevel::new(),
            business: BusinessMetrics::new(),
            narrative: NarrativeIntegrity::new(),
            budget: Budget::new(),
            political_capital: PoliticalCapital::new(),
            team: SecurityTeam::new(),
            compliance: ComplianceStatus::new(),
            threat_landscape: ThreatLandscape::new(),
            board,
            events,
            decisions_made: Vec::new(),
            active_incidents: Vec::new(),
            resolved_incidents: Vec::new(),
            phase: GamePhase::InheritanceDisaster,
            quarterly_objectives,
            technical_debt: TechnicalDebt::new(),
        }
    }

    fn initialize_board() -> Vec<BoardMember> {
        vec![
            BoardMember {
                role: BoardMemberRole::CEO,
                name: "Jennifer Walsh".to_string(),
                personality: BoardPersonality::PoliticallyShrewd,
                current_priority: BoardPriority::GrowthAtAllCosts,
                satisfaction: 70.0,
                influence: 95.0,
            },
            BoardMember {
                role: BoardMemberRole::CFO,
                name: "David Park".to_string(),
                personality: BoardPersonality::BottomLineFocused,
                current_priority: BoardPriority::CostReduction,
                satisfaction: 60.0,
                influence: 80.0,
            },
            BoardMember {
                role: BoardMemberRole::CTO,
                name: "Alex Thompson".to_string(),
                personality: BoardPersonality::TechnicallyMinded,
                current_priority: BoardPriority::RiskMitigation,
                satisfaction: 50.0,  // Skeptical of new CISO
                influence: 75.0,
            },
            BoardMember {
                role: BoardMemberRole::GeneralCounsel,
                name: "Maria Rodriguez".to_string(),
                personality: BoardPersonality::RiskAverse,
                current_priority: BoardPriority::ComplianceFirst,
                satisfaction: 55.0,
                influence: 70.0,
            },
        ]
    }

    fn initial_objectives(_board: &[BoardMember]) -> Vec<Objective> {
        vec![
            Objective {
                id: "soc2_cert".to_string(),
                description: "Achieve SOC2 Type II certification within 2 quarters".to_string(),
                assigned_quarter: 1,
                priority: ObjectivePriority::Critical,
                progress: 0.0,
                completion_turn: None,
                assigned_by: BoardMemberRole::CEO,
            },
            Objective {
                id: "reduce_incidents".to_string(),
                description: "Reduce security incidents by 40%".to_string(),
                assigned_quarter: 1,
                priority: ObjectivePriority::High,
                progress: 0.0,
                completion_turn: None,
                assigned_by: BoardMemberRole::CTO,
            },
        ]
    }

    pub fn add_event(&mut self, event_type: EventType, description: String, 
                     decision_id: Option<String>, visibility: EventVisibility) {
        let mut metadata = HashMap::new();
        metadata.insert("phase".to_string(), format!("{:?}", self.phase));
        metadata.insert("quarter".to_string(), self.quarter.to_string());

        self.events.push(Event {
            timestamp: Utc::now(),
            turn: self.turn,
            event_type,
            description,
            decision_id,
            visibility,
            metadata,
        });
    }

    pub fn advance_turn(&mut self) {
        self.turn += 1;
        
        // Natural processes
        self.risk.apply_decay(self.turn);
        self.risk.calculate_cascade_effects();
        self.threat_landscape.evolve(self.turn);
        self.technical_debt.total_debt_points += self.technical_debt.debt_velocity;
        
        // Check for team attrition
        let departed = self.team.check_attrition(self.turn);
        for name in departed {
            self.add_event(
                EventType::TeamMemberDeparted,
                format!("{} resigned. Exit interview cites: 'burnout', 'lack of resources', 'constant firefighting'", name),
                None,
                EventVisibility::Internal,
            );
            self.team.total_capacity -= 8.0;  // Losing someone hurts
            self.team.morale -= 10.0;
        }

        // Check for risk materialization
        let _materialized = self.check_risk_materialization();
        
        // Update phase
        self.phase = match self.turn {
            1..=3 => GamePhase::InheritanceDisaster,
            4..=12 => GamePhase::OperationalTempo,
            13..=16 => GamePhase::Discovery,
            _ => {
                let ending = self.calculate_ending();
                GamePhase::Ended(ending)
            }
        };

        // Quarter boundaries - THE MOST STRESSFUL MOMENTS
        if self.turn % 4 == 0 {
            self.conduct_quarterly_review();
        }
    }

    /// Quarterly review - where careers are made or ended
    fn conduct_quarterly_review(&mut self) {
        self.quarter += 1;
        
        self.add_event(
            EventType::QuarterEnd,
            format!("Q{} ends. Board review in progress...", self.quarter - 1),
            None,
            EventVisibility::Board,
        );

        // Reset political capital tracking
        self.political_capital.quarterly_reset();

        // Evaluate objectives
        let mut objectives_met = 0;
        let mut critical_objectives_missed = Vec::new();

        for objective in &mut self.quarterly_objectives {
            if objective.progress >= 100.0 && objective.completion_turn.is_none() {
                objective.completion_turn = Some(self.turn);
                objectives_met += 1;
            } else if objective.priority == ObjectivePriority::Critical && objective.progress < 50.0 {
                critical_objectives_missed.push(objective.description.clone());
            }
        }

        // Board member reactions
        let mut board_feedback = Vec::new();
        for member in &self.board {
            let reaction = self.evaluate_board_member_satisfaction(member);
            board_feedback.push(format!("{} ({}): {}", 
                member.name, 
                format!("{:?}", member.role).replace('_', " "),
                reaction
            ));
        }

        // Calculate political capital earned/lost
        let capital_change = if objectives_met > 0 {
            let gain = objectives_met as f64 * 10.0;
            self.political_capital.earn(gain, "Quarterly objectives met".to_string());
            gain
        } else {
            let loss = critical_objectives_missed.len() as f64 * 15.0;
            self.political_capital.total = (self.political_capital.total - loss).max(0.0);
            -loss
        };

        // Generate new objectives for next quarter
        self.generate_next_quarter_objectives();

        // Record review event
        self.add_event(
            EventType::BoardReview,
            format!(
                "Q{} Board Review:\n- Objectives met: {}\n- Critical misses: {}\n- Political capital: {:+.0}\n\nBoard feedback:\n{}",
                self.quarter - 1,
                objectives_met,
                critical_objectives_missed.len(),
                capital_change,
                board_feedback.join("\n")
            ),
            None,
            EventVisibility::Board,
        );
    }

    fn evaluate_board_member_satisfaction(&self, member: &BoardMember) -> String {
        match member.satisfaction {
            s if s > 80.0 => {
                match member.personality {
                    BoardPersonality::DataDriven => "Excellent metrics. Keep it up.".to_string(),
                    BoardPersonality::PoliticallyShrewd => "The board is impressed with your progress.".to_string(),
                    BoardPersonality::TechnicallyMinded => "Finally, someone who gets it.".to_string(),
                    BoardPersonality::BottomLineFocused => "ROI is acceptable.".to_string(),
                    BoardPersonality::RiskAverse => "I'm sleeping better at night.".to_string(),
                }
            }
            s if s > 50.0 => {
                match member.personality {
                    BoardPersonality::DataDriven => "Show me more data on your progress.".to_string(),
                    BoardPersonality::PoliticallyShrewd => "We need to discuss your approach.".to_string(),
                    BoardPersonality::TechnicallyMinded => "The technical debt concerns me.".to_string(),
                    BoardPersonality::BottomLineFocused => "Your budget utilization needs work.".to_string(),
                    BoardPersonality::RiskAverse => "I'm not comfortable with current risk levels.".to_string(),
                }
            }
            _ => {
                match member.personality {
                    BoardPersonality::DataDriven => "The numbers don't support your decisions.".to_string(),
                    BoardPersonality::PoliticallyShrewd => "We're hearing concerns from other stakeholders.".to_string(),
                    BoardPersonality::TechnicallyMinded => "This is amateur hour.".to_string(),
                    BoardPersonality::BottomLineFocused => "You're burning cash without results.".to_string(),
                    BoardPersonality::RiskAverse => "I'm updating my resume. You should too.".to_string(),
                }
            }
        }
    }

    fn generate_next_quarter_objectives(&mut self) {
        // Objectives get harder each quarter
        let _difficulty_multiplier = 1.0 + (self.quarter as f64 * 0.2);
        
        let new_objective = match self.quarter {
            2 => Objective {
                id: format!("q{}_objective", self.quarter),
                description: "Implement MFA for all administrative accounts".to_string(),
                assigned_quarter: self.quarter,
                priority: ObjectivePriority::High,
                progress: 0.0,
                completion_turn: None,
                assigned_by: BoardMemberRole::CTO,
            },
            3 => Objective {
                id: format!("q{}_objective", self.quarter),
                description: "Reduce mean time to detect (MTTD) to under 4 hours".to_string(),
                assigned_quarter: self.quarter,
                priority: ObjectivePriority::High,
                progress: 0.0,
                completion_turn: None,
                assigned_by: BoardMemberRole::CEO,
            },
            _ => Objective {
                id: format!("q{}_objective", self.quarter),
                description: "Maintain operational excellence".to_string(),
                assigned_quarter: self.quarter,
                priority: ObjectivePriority::Medium,
                progress: 0.0,
                completion_turn: None,
                assigned_by: BoardMemberRole::CEO,
            },
        };

        self.quarterly_objectives.push(new_objective);
    }

    pub fn calculate_ending(&self) -> Ending {
        let critical_incidents = self.active_incidents.iter()
            .filter(|i| matches!(i.severity, IncidentSeverity::Critical))
            .count();
        
        let unresolved_critical = self.active_incidents.iter()
            .filter(|i| matches!(i.severity, IncidentSeverity::Critical) 
                     && !matches!(i.response_status, IncidentResponseStatus::Closed))
            .count();
        
        let narrative_score = self.narrative.score;
        let business_health = self.business.arr_millions > 10.0 
            && self.business.board_confidence_percent > 50.0;
        let compliance_score = self.compliance.frameworks.get(&ComplianceFramework::SOC2)
            .map(|f| f.compliance_percent).unwrap_or(0.0);

        // Criminal investigation - you buried too much
        if self.narrative.criminal_exposure() {
            return Ending::CriminalInvestigation;
        }

        // Golden CISO - top 5%
        if critical_incidents == 0 
           && narrative_score > 85.0 
           && business_health 
           && self.risk.total_exposure < 150.0 
           && compliance_score > 90.0 
           && self.board.iter().all(|b| b.satisfaction > 70.0) {
            return Ending::GoldenCISO;
        }

        // Post-breach cleanup - bottom 25%
        if unresolved_critical > 0 
           || narrative_score < 50.0 
           || self.business.board_confidence_percent < 30.0 {
            return Ending::PostBreachCleanup;
        }

        // Lawsuit survivor - middle 70%
        Ending::LawsuitSurvivor
    }

    pub fn apply_decision_impact(&mut self, impact: &DecisionImpact) {
        // Risk changes
        self.risk.apply_delta(&impact.risk_delta);
        
        // Business changes
        self.business.apply_delta(&impact.business_delta);
        
        // Reputation changes
        let rep = &mut self.player.reputation;
        rep.industry_standing = (rep.industry_standing + impact.reputation_impact.industry_delta).max(0.0).min(100.0);
        rep.board_credibility = (rep.board_credibility + impact.reputation_impact.board_delta).max(0.0).min(100.0);
        rep.team_morale = (rep.team_morale + impact.reputation_impact.team_delta).max(0.0).min(100.0);
        rep.vendor_relationships = (rep.vendor_relationships + impact.reputation_impact.vendor_delta).max(0.0).min(100.0);


        // Team capacity
        if impact.team_capacity_required > 0.0 {
            self.team.allocate_capacity(impact.team_capacity_required);
        }

        // Political capital
        if impact.political_capital_cost > 0.0 {
            self.political_capital.spend(impact.political_capital_cost, None);
        }
        if impact.political_capital_gain > 0.0 {
            self.political_capital.earn(impact.political_capital_gain, impact.decision_id.clone());
        }

        // Budget
        if impact.budget_cost > 0.0 {
            self.budget.spend(impact.budget_cost, impact.budget_category);
        }

        // Compliance
        for (framework, progress) in &impact.compliance_impact.framework_progress {
            if let Some(status) = self.compliance.frameworks.get_mut(framework) {
                status.compliance_percent = (status.compliance_percent + progress).max(0.0).min(100.0);
            }
        }

        // Narrative integrity
        if let Some(narrative) = &impact.narrative_impact {
            self.narrative.score = (self.narrative.score - narrative.integrity_penalty).max(0.0);
            
            if narrative.creates_inconsistency {
                self.narrative.record_inconsistency(
                    self.turn,
                    narrative.reason.clone(),
                    narrative.integrity_penalty,
                );
            }

            if let Some((incident_id, actual_sev, reported_sev)) = &narrative.buries_incident {
                self.narrative.bury_incident(
                    incident_id.clone(),
                    *actual_sev,
                    *reported_sev,
                    self.turn,
                    narrative.reason.clone(),
                );
            }

            if let Some((incident_id, delay_turns)) = &narrative.delays_escalation {
                self.narrative.delay_escalation(
                    incident_id.clone(),
                    self.turn - delay_turns,
                    self.turn,
                    narrative.reason.clone(),
                );
            }
        }

        // Board member reactions
        for member in &mut self.board {
            member.react_to_decision(impact);
        }

        // Record decision
        self.decisions_made.push(impact.decision_id.clone());
    }

    pub fn trigger_incident(&mut self, incident: ActiveIncident) {
        let visibility = if incident.severity == IncidentSeverity::Critical {
            EventVisibility::Board
        } else {
            EventVisibility::Internal
        };

        self.add_event(
            EventType::IncidentDetected,
            format!("Incident detected: {} [{}]", incident.title, format!("{:?}", incident.severity)),
            None,
            visibility,
        );

        // Consume team capacity for incident response
        let capacity_needed = match incident.severity {
            IncidentSeverity::Critical => 8.0,
            IncidentSeverity::High => 5.0,
            IncidentSeverity::Medium => 3.0,
            IncidentSeverity::Low => 1.0,
        };

        if !self.team.allocate_capacity(capacity_needed) {
            // Team is at capacity - incident will get worse
            self.add_event(
                EventType::IncidentDetected,
                "WARNING: Insufficient team capacity for proper incident response".to_string(),
                None,
                EventVisibility::Internal,
            );
        }

        self.active_incidents.push(incident);
    }

    /// Check if delayed risk should materialize - now more sophisticated
    pub fn check_risk_materialization(&mut self) -> Vec<String> {
        let mut materialized = Vec::new();
        
        // Data exposure risk with time-to-critical
        if let Some(data_metric) = self.risk.vectors.get(&RiskVector::DataExposure) {
            if data_metric.current_level > 60.0 && self.turn > 5 {
                if !self.active_incidents.iter().any(|i| i.id == "s3_breach") {
                    let incident = ActiveIncident {
                        id: "s3_breach".to_string(),
                        title: "S3 Bucket Public Exposure".to_string(),
                        description: "S3 bucket containing customer PII found publicly accessible. Misconfigured 8 months ago during migration.".to_string(),
                        severity: IncidentSeverity::Critical,
                        turn_detected: self.turn,
                        turn_deadline: Some(self.turn + 2),  // 2 turns before this goes public
                        escalated_to_board: false,
                        escalation_turn: None,
                        response_status: IncidentResponseStatus::Detected,
                        assigned_team: Vec::new(),
                        capacity_consumed: 0.0,
                        containment_percent: 0.0,
                        root_cause_identified: false,
                        public_disclosure_required: true,
                        customer_impact_count: Some(840000),
                        timeline: vec![
                            IncidentTimelineEntry {
                                turn: self.turn,
                                action: "Bucket discovered publicly accessible via automated scan".to_string(),
                                actor: "Security tooling".to_string(),
                                visibility: EventVisibility::Internal,
                            }
                        ],
                    };
                    self.trigger_incident(incident);
                    materialized.push("CRITICAL: S3 bucket with 840K customer records publicly exposed".to_string());
                }
            }
        }

        // Access control with credential stuffing
        if let Some(access_metric) = self.risk.vectors.get(&RiskVector::AccessControl) {
            if access_metric.current_level > 50.0 && access_metric.mitigation_coverage < 30.0 && self.turn > 6 {
                if !self.active_incidents.iter().any(|i| i.id == "credential_stuffing") {
                    let incident = ActiveIncident {
                        id: "credential_stuffing".to_string(),
                        title: "Admin Account Compromise".to_string(),
                        description: "Credential stuffing attack successful on admin accounts. No MFA. Attacker accessed production systems.".to_string(),
                        severity: IncidentSeverity::High,
                        turn_detected: self.turn,
                        turn_deadline: Some(self.turn + 3),
                        escalated_to_board: false,
                        escalation_turn: None,
                        response_status: IncidentResponseStatus::Detected,
                        assigned_team: Vec::new(),
                        capacity_consumed: 0.0,
                        containment_percent: 0.0,
                        root_cause_identified: false,
                        public_disclosure_required: false,
                        customer_impact_count: None,
                        timeline: vec![
                            IncidentTimelineEntry {
                                turn: self.turn,
                                action: "Suspicious admin logins detected from unusual IP ranges".to_string(),
                                actor: "SIEM alert".to_string(),
                                visibility: EventVisibility::Internal,
                            }
                        ],
                    };
                    self.trigger_incident(incident);
                    materialized.push("HIGH: Admin account compromised via credential stuffing".to_string());
                }
            }
        }

        // Vendor risk cascading
        if let Some(vendor_metric) = self.risk.vectors.get(&RiskVector::VendorRisk) {
            if vendor_metric.current_level > 40.0 && self.turn > 7 {
                if !self.active_incidents.iter().any(|i| i.id == "vendor_breach") {
                    let incident = ActiveIncident {
                        id: "vendor_breach".to_string(),
                        title: "Third-Party SSO Provider Breach".to_string(),
                        description: "SSO provider disclosed breach. Unknown if customer credentials compromised. Vendor is being 'less than forthcoming'.".to_string(),
                        severity: IncidentSeverity::High,
                        turn_detected: self.turn,
                        turn_deadline: Some(self.turn + 4),
                        escalated_to_board: false,
                        escalation_turn: None,
                        response_status: IncidentResponseStatus::Investigating,
                        assigned_team: Vec::new(),
                        capacity_consumed: 0.0,
                        containment_percent: 0.0,
                        root_cause_identified: false,
                        public_disclosure_required: true,
                        customer_impact_count: None,
                        timeline: vec![
                            IncidentTimelineEntry {
                                turn: self.turn,
                                action: "Vendor notification received via email (not phone call - red flag)".to_string(),
                                actor: "Vendor".to_string(),
                                visibility: EventVisibility::Internal,
                            }
                        ],
                    };
                    self.trigger_incident(incident);
                    materialized.push("HIGH: SSO vendor breach - impact assessment needed".to_string());
                }
            }
        }

        // Technical debt causing incidents
        if self.technical_debt.total_debt_points > 200.0 && self.turn % 3 == 0 {
            if !self.active_incidents.iter().any(|i| i.id.starts_with("debt_incident")) {
                let incident = ActiveIncident {
                    id: format!("debt_incident_{}", self.turn),
                    title: "Legacy System Vulnerability Exploited".to_string(),
                    description: "Unpatched system from 2019 compromised. 'We were going to fix that next quarter' - famous last words.".to_string(),
                    severity: IncidentSeverity::Medium,
                    turn_detected: self.turn,
                    turn_deadline: Some(self.turn + 2),
                    escalated_to_board: false,
                    escalation_turn: None,
                    response_status: IncidentResponseStatus::Detected,
                    assigned_team: Vec::new(),
                    capacity_consumed: 0.0,
                    containment_percent: 0.0,
                    root_cause_identified: true,  // Oh, we know exactly what happened
                    public_disclosure_required: false,
                    customer_impact_count: None,
                    timeline: Vec::new(),
                };
                self.trigger_incident(incident);
                materialized.push("MEDIUM: Technical debt materialized - legacy system compromised".to_string());
            }
        }

        materialized
    }

    /// Alias for check_risk_materialization - more intuitive naming
    pub fn materialize_risks(&mut self) -> Vec<String> {
        self.check_risk_materialization()
    }

    /// Escalate incident to board - this is a BIG decision
    pub fn escalate_incident_to_board(&mut self, incident_id: &str) -> Result<()> {
        // Extract data we need BEFORE any mutable operations
        let (turn_detected, incident_title, _already_escalated) = {
            let incident = self.active_incidents.iter()
                .find(|i| i.id == incident_id)
                .ok_or(GameError::InvalidAction)?;
            
            if incident.escalated_to_board {
                return Err(GameError::InvalidAction);
            }
            
            (incident.turn_detected, incident.title.clone(), incident.escalated_to_board)
        };
        
        let is_timely = self.turn - turn_detected <= 1;
        
        // Now do all mutable operations without any borrows
        if is_timely {
            self.political_capital.earn(5.0, "Proactive escalation".to_string());
            self.add_event(
                EventType::IncidentEscalated,
                format!("Board appreciates proactive notification of {}", incident_title),
                None,
                EventVisibility::Board,
            );
        } else {
            self.political_capital.total = (self.political_capital.total - 10.0).max(0.0);
            self.add_event(
                EventType::IncidentEscalated,
                format!("Board questions delay in escalating {}", incident_title),
                None,
                EventVisibility::Board,
            );
            
            // Create narrative inconsistency
            let delay = self.turn - turn_detected;
            self.narrative.delay_escalation(
                incident_id.to_string(),
                turn_detected,
                self.turn,
                format!("Delayed {} turns before board notification", delay),
            );
        }

        // Finally, update the incident itself
        let incident = self.active_incidents.iter_mut()
            .find(|i| i.id == incident_id)
            .ok_or(GameError::InvalidAction)?;
            
        incident.escalated_to_board = true;
        incident.escalation_turn = Some(self.turn);
        incident.timeline.push(IncidentTimelineEntry {
            turn: self.turn,
            action: "Incident escalated to board".to_string(),
            actor: self.player.name.clone(),
            visibility: EventVisibility::Board,
        });


        Ok(())
    }

    /// Resolve incident - requires work and leaves a trail
    pub fn resolve_incident(&mut self, incident_id: &str, lessons_learned: Vec<String>) -> Result<()> {
        let incident_index = self.active_incidents.iter()
            .position(|i| i.id == incident_id)
            .ok_or(GameError::InvalidAction)?;

        let incident = self.active_incidents.remove(incident_index);
        
        let time_to_resolve = self.turn - incident.turn_detected;
        let final_cost = match incident.severity {
            IncidentSeverity::Critical => 0.5,  // $500K
            IncidentSeverity::High => 0.2,
            IncidentSeverity::Medium => 0.05,
            IncidentSeverity::Low => 0.01,
        };

        // Reputation impact
        let rep_impact = if incident.public_disclosure_required {
            -20.0
        } else if incident.escalated_to_board {
            -5.0
        } else {
            0.0
        };

        let resolved = ResolvedIncident {
            id: format!("resolved_{}", incident.id),
            original_incident: incident.id.clone(),
            resolution_turn: self.turn,
            time_to_resolve,
            lessons_learned: lessons_learned.clone(),
            follow_up_actions: vec![
                "Update runbooks".to_string(),
                "Implement additional controls".to_string(),
                "Schedule post-mortem review".to_string(),
            ],
            final_cost,
            reputation_impact: rep_impact,
        };

        // Update team morale based on how it went
        if time_to_resolve <= 3 {
            self.team.morale = (self.team.morale + 5.0).min(100.0);
        } else {
            self.team.morale = (self.team.morale - 5.0).max(0.0);
        }

        // Budget impact
        self.budget.spend(final_cost, BudgetCategory::Emergency);

        self.resolved_incidents.push(resolved);

        self.add_event(
            EventType::IncidentResolved,
            format!("Incident {} resolved after {} turns. Lessons learned: {}", 
                    incident.title, time_to_resolve, lessons_learned.join(", ")),
            None,
            if incident.escalated_to_board { EventVisibility::Board } else { EventVisibility::Internal },
        );

        Ok(())
    }
}