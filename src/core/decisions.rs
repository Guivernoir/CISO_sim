use crate::core::types::*;
use crate::core::state::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A decision point in the game - where careers are made or broken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub turn: u32,
    pub title: String,
    pub context: String,
    pub choices: Vec<Choice>,
    pub is_board_pressure: bool,
    pub is_time_sensitive: bool,
    pub decision_category: DecisionCategory,
    pub prerequisites: Vec<String>,  // Required prior decisions/conditions
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DecisionCategory {
    StrategicDirection,
    IncidentResponse,
    BudgetAllocation,
    ComplianceApproach,
    TeamManagement,
    VendorSelection,
    RiskAcceptance,
    PoliticalNavigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub description: String,
    pub impact_preview: ImpactPreview,
    pub impact_data: Option<DecisionImpact>,
    pub prerequisites: ChoicePrerequisites,
    pub consequences: Vec<DelayedConsequence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoicePrerequisites {
    pub min_budget: f64,
    pub min_political_capital: f64,
    pub min_team_capacity: f64,
    pub required_compliance: Vec<ComplianceFramework>,
    pub blocked_by: Vec<String>,  // Can't choose if these decisions were made
}

impl Default for ChoicePrerequisites {
    fn default() -> Self {
        Self {
            min_budget: 0.0,
            min_political_capital: 0.0,
            min_team_capacity: 0.0,
            required_compliance: Vec::new(),
            blocked_by: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayedConsequence {
    pub trigger_turn: u32,
    pub event_type: EventType,
    pub description: String,
    pub additional_impact: Option<DecisionImpact>,
}

/// What the player sees before making the choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPreview {
    pub estimated_arr_change: f64,
    pub budget_cost: f64,
    pub timeline_weeks: Option<u32>,
    pub political_note: Option<String>,
    pub risk_indicator: RiskIndicator,
    pub compliance_impact: ComplianceImpact,
    pub team_impact: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskIndicator {
    Reduces,       // Green
    Neutral,       // Yellow
    Increases,     // Orange
    Significant,   // Red
}

impl Decision {
    /// Apply a chosen option to the game state, returning the full impact
    pub fn apply_choice(&mut self, choice_id: &str, state: &mut GameState) -> Result<DecisionImpact> {
        // Find the choice
        let choice = self.choices.iter()
            .find(|c| c.id == choice_id)
            .ok_or(GameError::InvalidAction)?;
        
        // Check prerequisites
        if choice.prerequisites.min_budget > 0.0 && state.budget.available() < choice.prerequisites.min_budget {
            return Err(GameError::InsufficientBudget);
        }
        
        if choice.prerequisites.min_political_capital > 0.0 && 
           state.political_capital.total < choice.prerequisites.min_political_capital {
            return Err(GameError::InsufficientPoliticalCapital);
        }
        
        if choice.prerequisites.min_team_capacity > 0.0 &&
           state.team.available_capacity() < choice.prerequisites.min_team_capacity {
            return Err(GameError::TeamCapacityExceeded);
        }
        
        // Get the full impact data
        let impact = choice.impact_data.clone()
            .unwrap_or_else(|| DecisionImpact::new(choice.id.clone()));
        
        // Apply the impact to state
        state.risk.apply_delta(&impact.risk_delta);
        state.business.apply_delta(&impact.business_delta);
        
        // Handle budget
        if impact.budget_cost > 0.0 {
            if !state.budget.spend(impact.budget_cost, impact.budget_category) {
                return Err(GameError::InsufficientBudget);
            }
        }
        
        // Handle political capital
        if impact.political_capital_cost > 0.0 {
            if !state.political_capital.spend(impact.political_capital_cost, None) {
                return Err(GameError::InsufficientPoliticalCapital);
            }
        }
        if impact.political_capital_gain > 0.0 {
            state.political_capital.earn(impact.political_capital_gain, format!("Decision: {}", self.title));
        }
        
        // Handle team capacity
        if impact.team_capacity_required > 0.0 {
            if !state.team.allocate_capacity(impact.team_capacity_required) {
                return Err(GameError::TeamCapacityExceeded);
            }
        }
        
        // Apply reputation changes
        state.player.reputation.industry_standing += impact.reputation_impact.industry_delta;
        state.player.reputation.board_credibility += impact.reputation_impact.board_delta;
        state.player.reputation.team_morale += impact.reputation_impact.team_delta;
        state.player.reputation.vendor_relationships += impact.reputation_impact.vendor_delta;
        
        // Apply compliance impact
        for (framework, progress) in &impact.compliance_impact.framework_progress {
            if let Some(status) = state.compliance.frameworks.get_mut(framework) {
                status.compliance_percent += progress;
            }
        }
        
        // Apply narrative impact
        if let Some(ref narrative_impact) = impact.narrative_impact {
            // Directly modify narrative integrity score
            state.narrative.score = (state.narrative.score - narrative_impact.integrity_penalty).max(0.0);
            
            if let Some((inc_id, actual_sev, reported_sev)) = &narrative_impact.buries_incident {
                state.narrative.bury_incident(
                    inc_id.clone(),
                    *actual_sev,
                    *reported_sev,
                    state.turn,
                    narrative_impact.reason.clone(),
                );
            }
            
            if let Some((inc_id, delay_turns)) = &narrative_impact.delays_escalation {
                state.narrative.delay_escalation(
                    inc_id.clone(),
                    state.turn,
                    state.turn + delay_turns,
                    narrative_impact.reason.clone(),
                );
            }
        }
        
        // Record the decision
        state.decisions_made.push(self.id.clone());
        state.add_event(
            EventType::DecisionMade,
            format!("Decision: {} - Chose: {}", self.title, choice.label),
            Some(self.id.clone()),
            EventVisibility::Management,
        );
        
        Ok(impact)
    }
}

/// Decision factory - creates the tough calls
pub struct DecisionFactory;

impl DecisionFactory {
    /// Generate decisions based on game state
    pub fn generate_decision(state: &GameState) -> Option<Decision> {
        match state.turn {
            1 => Some(Self::turn_1_inheritance_decision()),
            2 => Some(Self::turn_2_triage_decision(state)),
            3 => Some(Self::turn_3_quick_win_or_foundation()),
            5 => Self::generate_incident_decision(state),
            6 => Some(Self::compliance_pressure_decision(state)),
            8 => Some(Self::budget_battle_decision(state)),
            10 => Some(Self::team_crisis_decision(state)),
            12 => Some(Self::vendor_selection_decision()),
            14 => Self::generate_discovery_decision(state),
            _ => Self::generate_dynamic_decision(state),
        }
    }

    fn turn_1_inheritance_decision() -> Decision {
        Decision {
            id: "turn_1_inheritance".to_string(),
            turn: 1,
            title: "The Inheritance".to_string(),
            context: "You've just started as CSO. Your first security review reveals: \n\
                     - No MFA on admin accounts\n\
                     - 847 unpatched servers\n\
                     - SOC2 audit in 60 days\n\
                     - Previous CSO's documentation: 'Good luck'\n\n\
                     The CEO is asking: 'What's your plan?'".to_string(),
            choices: vec![
                Choice {
                    id: "honest_assessment".to_string(),
                    label: "Honest Assessment".to_string(),
                    description: "Tell the board exactly how bad it is. Request emergency budget and timeline extension.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -0.5,
                        budget_cost: 0.0,
                        timeline_weeks: Some(12),
                        political_note: Some("Board may question your competence immediately".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team appreciates transparency".to_string(),
                    },
                    impact_data: Some(Self::honest_assessment_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![],
                },
                Choice {
                    id: "optimistic_commitment".to_string(),
                    label: "Optimistic Commitment".to_string(),
                    description: "Promise to have everything fixed by SOC2 audit. We can make this work... right?".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.0,
                        timeline_weeks: Some(8),
                        political_note: Some("Board loves confidence, but...".to_string()),
                        risk_indicator: RiskIndicator::Significant,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team is already stressed".to_string(),
                    },
                    impact_data: Some(Self::optimistic_commitment_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![
                        DelayedConsequence {
                            trigger_turn: 8,
                            event_type: EventType::ComplianceAudit,
                            description: "SOC2 audit reveals gaps you promised were fixed".to_string(),
                            additional_impact: Some(Self::audit_failure_impact()),
                        }
                    ],
                },
                Choice {
                    id: "selective_disclosure".to_string(),
                    label: "Selective Disclosure".to_string(),
                    description: "Share the 'top priorities' but don't mention everything. Buy time to fix things quietly.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.0,
                        timeline_weeks: Some(10),
                        political_note: Some("Balanced approach".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team uncertain about direction".to_string(),
                    },
                    impact_data: Some(Self::selective_disclosure_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![],
                },
            ],
            is_board_pressure: true,
            is_time_sensitive: true,
            decision_category: DecisionCategory::StrategicDirection,
            prerequisites: Vec::new(),
        }
    }

    fn turn_2_triage_decision(_state: &GameState) -> Decision {
        Decision {
            id: "turn_2_triage".to_string(),
            turn: 2,
            title: "Triage: What Burns First?".to_string(),
            context: "Your team has bandwidth for ONE major initiative this quarter. Choose wisely.\n\
                     - MFA rollout (prevents credential attacks)\n\
                     - Patch management (847 servers at risk)\n\
                     - SOC2 documentation (audit approaching)\n\
                     - Detection capability (currently blind to attacks)".to_string(),
            choices: vec![
                Choice {
                    id: "mfa_priority".to_string(),
                    label: "MFA Rollout".to_string(),
                    description: "Implement MFA on all critical accounts. Will slow down sales team.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -0.3,
                        budget_cost: 0.15,
                        timeline_weeks: Some(6),
                        political_note: Some("Sales will complain loudly".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 15.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Clear, achievable goal".to_string(),
                    },
                    impact_data: Some(Self::mfa_priority_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.15,
                        min_team_capacity: 10.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "patch_priority".to_string(),
                    label: "Patch All The Things".to_string(),
                    description: "Emergency patching sprint. Will cause outages.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -0.5,
                        budget_cost: 0.1,
                        timeline_weeks: Some(8),
                        political_note: Some("Engineering will hate you".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 10.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Stressful but necessary".to_string(),
                    },
                    impact_data: Some(Self::patch_priority_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.1,
                        min_team_capacity: 12.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "soc2_documentation".to_string(),
                    label: "SOC2 Documentation Sprint".to_string(),
                    description: "Document everything. Hope nothing breaks before audit.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.2,
                        budget_cost: 0.05,
                        timeline_weeks: Some(4),
                        political_note: Some("Board will appreciate compliance focus".to_string()),
                        risk_indicator: RiskIndicator::Increases,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 30.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Death by documentation".to_string(),
                    },
                    impact_data: Some(Self::soc2_docs_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.05,
                        min_team_capacity: 8.0,
                        ..Default::default()
                    },
                    consequences: vec![
                        DelayedConsequence {
                            trigger_turn: 5,
                            event_type: EventType::RiskMaterialized,
                            description: "Unpatched vulnerability exploited during documentation sprint".to_string(),
                            additional_impact: Some(Self::deferred_risk_impact()),
                        }
                    ],
                },
            ],
            is_board_pressure: false,
            is_time_sensitive: true,
            decision_category: DecisionCategory::StrategicDirection,
            prerequisites: Vec::new(),
        }
    }

    fn turn_3_quick_win_or_foundation() -> Decision {
        Decision {
            id: "turn_3_foundation".to_string(),
            turn: 3,
            title: "Quick Win vs. Foundation".to_string(),
            context: "The board wants to see 'visible progress'. Your team needs proper tooling and process.\n\
                     Classic CSO dilemma: theater or substance?".to_string(),
            choices: vec![
                Choice {
                    id: "security_theater".to_string(),
                    label: "Security Theater".to_string(),
                    description: "Deploy fancy security dashboard for board. Looks great, does little.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.08,
                        timeline_weeks: Some(2),
                        political_note: Some("Board loves dashboards".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team morale decreases".to_string(),
                    },
                    impact_data: Some(Self::security_theater_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.08,
                        min_political_capital: 10.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "build_foundation".to_string(),
                    label: "Build Foundation".to_string(),
                    description: "Implement proper SIEM, logging, alerting. Takes time but actually works.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.25,
                        timeline_weeks: Some(12),
                        political_note: Some("Hard to explain value to non-technical board".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 20.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team energized by real work".to_string(),
                    },
                    impact_data: Some(Self::build_foundation_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.25,
                        min_team_capacity: 15.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
            ],
            is_board_pressure: true,
            is_time_sensitive: false,
            decision_category: DecisionCategory::BudgetAllocation,
            prerequisites: Vec::new(),
        }
    }

    fn compliance_pressure_decision(state: &GameState) -> Decision {
        let soc2_progress = state.compliance.frameworks.get(&ComplianceFramework::SOC2)
            .map(|f| f.compliance_percent).unwrap_or(0.0);

        Decision {
            id: "turn_6_compliance".to_string(),
            turn: 6,
            title: "The Auditor Cometh".to_string(),
            context: format!(
                "SOC2 audit is in 2 turns. Current compliance: {:.0}%.\n\
                 Auditor's preliminary findings: 'Material weaknesses in access control and change management'.\n\
                 CFO: 'We NEED this certification to close the Series B.'",
                soc2_progress
            ),
            choices: vec![
                Choice {
                    id: "emergency_remediation".to_string(),
                    label: "Emergency Remediation".to_string(),
                    description: "All hands on deck. Fix the findings. Team will burn out.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.3,
                        budget_cost: 0.15,
                        timeline_weeks: Some(3),
                        political_note: Some("High risk, high reward".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 40.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Burnout risk: High".to_string(),
                    },
                    impact_data: Some(Self::emergency_remediation_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.15,
                        min_team_capacity: 18.0,
                        ..Default::default()
                    },
                    consequences: vec![
                        DelayedConsequence {
                            trigger_turn: 10,
                            event_type: EventType::TeamMemberDeparted,
                            description: "Senior engineer quits citing burnout".to_string(),
                            additional_impact: Some(Self::burnout_impact()),
                        }
                    ],
                },
                Choice {
                    id: "negotiate_timeline".to_string(),
                    label: "Negotiate Timeline".to_string(),
                    description: "Request audit extension. Honest about current state. Series B may be delayed.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -0.4,
                        budget_cost: 0.05,
                        timeline_weeks: Some(16),
                        political_note: Some("CFO will not be happy".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 10.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Sustainable pace".to_string(),
                    },
                    impact_data: Some(Self::negotiate_timeline_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_political_capital: 20.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "paper_over_gaps".to_string(),
                    label: "Paper Over Gaps".to_string(),
                    description: "Creative documentation. Technical debt hidden. Audit passes... for now.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.5,
                        budget_cost: 0.05,
                        timeline_weeks: Some(2),
                        political_note: Some("What could go wrong?".to_string()),
                        risk_indicator: RiskIndicator::Significant,
                        compliance_impact: ComplianceImpact {
                            framework_progress: {
                                let mut progress = HashMap::new();
                                progress.insert(ComplianceFramework::SOC2, 50.0);
                                progress
                            },
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Moral hazard".to_string(),
                    },
                    impact_data: Some(Self::paper_over_gaps_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![
                        DelayedConsequence {
                            trigger_turn: 14,
                            event_type: EventType::ComplianceAudit,
                            description: "Re-audit discovers falsified documentation. Criminal referral considered.".to_string(),
                            additional_impact: Some(Self::fraud_discovered_impact()),
                        }
                    ],
                },
            ],
            is_board_pressure: true,
            is_time_sensitive: true,
            decision_category: DecisionCategory::ComplianceApproach,
            prerequisites: Vec::new(),
        }
    }

    fn budget_battle_decision(state: &GameState) -> Decision {
        Decision {
            id: "turn_8_budget".to_string(),
            turn: 8,
            title: "Budget Battle: Q3 Planning".to_string(),
            context: format!(
                "Q3 budget planning. CFO: 'Security spent ${:.1}M last quarter. Show me ROI.'\n\
                 Your current budget: ${:.2}M remaining.\n\
                 Requests: New security engineer ($150K), SIEM upgrade ($200K), Compliance tool ($100K)",
                state.budget.spent, state.budget.available()
            ),
            choices: vec![
                Choice {
                    id: "fight_for_budget".to_string(),
                    label: "Fight for Full Budget".to_string(),
                    description: "Present risk metrics, industry benchmarks. Request full $450K.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.45,
                        timeline_weeks: None,
                        political_note: Some("CFO will remember this".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Can finally staff properly".to_string(),
                    },
                    impact_data: Some(Self::fight_for_budget_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_political_capital: 30.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "compromise_budget".to_string(),
                    label: "Strategic Compromise".to_string(),
                    description: "Accept engineer + compliance tool. Defer SIEM upgrade.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.25,
                        timeline_weeks: None,
                        political_note: Some("Reasonable approach".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Partial win".to_string(),
                    },
                    impact_data: Some(Self::compromise_budget_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![],
                },
                Choice {
                    id: "accept_cuts".to_string(),
                    label: "Accept Budget Cuts".to_string(),
                    description: "'We'll make do with less.' Preserve political capital.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.0,
                        timeline_weeks: None,
                        political_note: Some("CFO is pleased".to_string()),
                        risk_indicator: RiskIndicator::Increases,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Morale tanks".to_string(),
                    },
                    impact_data: Some(Self::accept_cuts_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![],
                },
            ],
            is_board_pressure: true,
            is_time_sensitive: false,
            decision_category: DecisionCategory::BudgetAllocation,
            prerequisites: Vec::new(),
        }
    }

    fn team_crisis_decision(state: &GameState) -> Decision {
        Decision {
            id: "turn_10_team".to_string(),
            turn: 10,
            title: "Team Crisis".to_string(),
            context: format!(
                "Your best engineer just got an offer: 30% raise, better work-life balance.\n\
                 Current team morale: {:.0}%\n\
                 Team capacity: {:.0}/{:.0} story points\n\
                 Losing them now would be devastating.",
                state.team.morale, 
                state.team.total_capacity - state.team.committed_capacity,
                state.team.total_capacity
            ),
            choices: vec![
                Choice {
                    id: "counter_offer".to_string(),
                    label: "Counter Offer".to_string(),
                    description: "Match the offer. Exceeds salary band. Sets precedent.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.15,
                        timeline_weeks: None,
                        political_note: Some("HR will push back".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Keeps the engineer, sets expectations".to_string(),
                    },
                    impact_data: Some(Self::counter_offer_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.15,
                        min_political_capital: 25.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "let_them_go".to_string(),
                    label: "Let Them Go".to_string(),
                    description: "Wish them well. Begin painful hiring process.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.0,
                        timeline_weeks: Some(12),
                        political_note: Some("Hiring takes 3+ months".to_string()),
                        risk_indicator: RiskIndicator::Increases,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Remaining team stressed".to_string(),
                    },
                    impact_data: Some(Self::let_them_go_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![
                        DelayedConsequence {
                            trigger_turn: 12,
                            event_type: EventType::RiskMaterialized,
                            description: "Short-staffed team misses critical alert".to_string(),
                            additional_impact: Some(Self::understaffed_impact()),
                        }
                    ],
                },
            ],
            is_board_pressure: false,
            is_time_sensitive: true,
            decision_category: DecisionCategory::TeamManagement,
            prerequisites: Vec::new(),
        }
    }

    fn vendor_selection_decision() -> Decision {
        Decision {
            id: "turn_12_vendor".to_string(),
            turn: 12,
            title: "Vendor Selection: Choose Your Poison".to_string(),
            context: "Need to select an EDR vendor. Sales teams are... aggressive.\n\
                     - Vendor A: Market leader, expensive, sales rep is CEO's golfing buddy\n\
                     - Vendor B: Best technology, reasonable price, unknown brand\n\
                     - Vendor C: Cheapest option, adequate features, questionable support".to_string(),
            choices: vec![
                Choice {
                    id: "political_choice".to_string(),
                    label: "Vendor A (Political Choice)".to_string(),
                    description: "Go with CEO's preferred vendor. Expensive but safe politically.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.35,
                        timeline_weeks: Some(8),
                        political_note: Some("CEO will be pleased".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team accepts it".to_string(),
                    },
                    impact_data: Some(Self::political_vendor_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.35,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "technical_choice".to_string(),
                    label: "Vendor B (Technical Choice)".to_string(),
                    description: "Choose the best technical solution. Prepare to explain to CEO.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.20,
                        timeline_weeks: Some(10),
                        political_note: Some("Will need to justify this".to_string()),
                        risk_indicator: RiskIndicator::Reduces,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team respects the decision".to_string(),
                    },
                    impact_data: Some(Self::technical_vendor_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.20,
                        min_political_capital: 20.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
                Choice {
                    id: "budget_choice".to_string(),
                    label: "Vendor C (Budget Choice)".to_string(),
                    description: "Cheap and adequate. What could go wrong?".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.10,
                        timeline_weeks: Some(6),
                        political_note: Some("CFO loves saving money".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team is skeptical".to_string(),
                    },
                    impact_data: Some(Self::budget_vendor_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.10,
                        ..Default::default()
                    },
                    consequences: vec![
                        DelayedConsequence {
                            trigger_turn: 15,
                            event_type: EventType::IncidentDetected,
                            description: "EDR fails to detect ransomware. Vendor support is... lacking.".to_string(),
                            additional_impact: Some(Self::vendor_failure_impact()),
                        }
                    ],
                },
            ],
            is_board_pressure: false,
            is_time_sensitive: false,
            decision_category: DecisionCategory::VendorSelection,
            prerequisites: Vec::new(),
        }
    }

    fn generate_incident_decision(state: &GameState) -> Option<Decision> {
        // Generate decision based on active incidents
        if let Some(incident) = state.active_incidents.first() {
            Some(Self::incident_response_decision(incident))
        } else {
            None
        }
    }

    fn incident_response_decision(incident: &ActiveIncident) -> Decision {
        Decision {
            id: format!("incident_{}", incident.id),
            turn: incident.turn_detected,
            title: format!("Incident Response: {}", incident.title),
            context: format!(
                "{}\n\nSeverity: {:?}\nTime pressure: {}",
                incident.description,
                incident.severity,
                if incident.turn_deadline.is_some() { "HIGH" } else { "MODERATE" }
            ),
            choices: vec![
                Choice {
                    id: "immediate_escalation".to_string(),
                    label: "Immediate Board Escalation".to_string(),
                    description: "Inform board immediately. CYA mode engaged.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -0.2,
                        budget_cost: 0.0,
                        timeline_weeks: None,
                        political_note: Some("Transparent but panic-inducing".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team under spotlight".to_string(),
                    },
                    impact_data: Some(Self::immediate_escalation_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![],
                },
                Choice {
                    id: "contain_first".to_string(),
                    label: "Contain First, Report Later".to_string(),
                    description: "Fix it quietly, then report with solution. Risky if it leaks.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: 0.0,
                        budget_cost: 0.05,
                        timeline_weeks: Some(1),
                        political_note: Some("Better optics, but...".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Team can work without interference".to_string(),
                    },
                    impact_data: Some(Self::contain_first_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_team_capacity: 8.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
            ],
            is_board_pressure: true,
            is_time_sensitive: true,
            decision_category: DecisionCategory::IncidentResponse,
            prerequisites: Vec::new(),
        }
    }

    fn generate_discovery_decision(state: &GameState) -> Option<Decision> {
        // Discovery phase - past decisions come back
        if state.narrative.score < 70.0 {
            Some(Self::discovery_phase_decision(state))
        } else {
            None
        }
    }

    fn discovery_phase_decision(state: &GameState) -> Decision {
        Decision {
            id: "discovery_reckoning".to_string(),
            turn: 14,
            title: "Discovery: The Reckoning".to_string(),
            context: format!(
                "External law firm conducting pre-IPO due diligence.\n\
                 They've found: inconsistencies in your incident reporting.\n\
                 Narrative integrity: {:.0}%\n\n\
                 Lead attorney: 'We need to discuss some... discrepancies.'",
                state.narrative.score
            ),
            choices: vec![
                Choice {
                    id: "full_disclosure".to_string(),
                    label: "Full Disclosure".to_string(),
                    description: "Tell them everything. Volunteer the skeletons. Clear conscience, unclear future.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -1.0,
                        budget_cost: 0.0,
                        timeline_weeks: None,
                        political_note: Some("IPO may be delayed".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Uncertain".to_string(),
                    },
                    impact_data: Some(Self::full_disclosure_impact()),
                    prerequisites: ChoicePrerequisites::default(),
                    consequences: vec![],
                },
                Choice {
                    id: "controlled_narrative".to_string(),
                    label: "Controlled Narrative".to_string(),
                    description: "Provide context, minimize damage. Lawyer up properly.".to_string(),
                    impact_preview: ImpactPreview {
                        estimated_arr_change: -0.3,
                        budget_cost: 0.15,
                        timeline_weeks: None,
                        political_note: Some("Damage control mode".to_string()),
                        risk_indicator: RiskIndicator::Neutral,
                        compliance_impact: ComplianceImpact {
                            framework_progress: HashMap::new(),
                            new_findings: Vec::new(),
                            resolved_findings: Vec::new(),
                        },
                        team_impact: "Professional approach".to_string(),
                    },
                    impact_data: Some(Self::controlled_narrative_impact()),
                    prerequisites: ChoicePrerequisites {
                        min_budget: 0.15,
                        min_political_capital: 15.0,
                        ..Default::default()
                    },
                    consequences: vec![],
                },
            ],
            is_board_pressure: true,
            is_time_sensitive: true,
            decision_category: DecisionCategory::PoliticalNavigation,
            prerequisites: Vec::new(),
        }
    }

    fn generate_dynamic_decision(_state: &GameState) -> Option<Decision> {
        // Generate decisions based on current state
        None  // Placeholder for dynamic generation
    }

    // Impact implementations
    fn honest_assessment_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("honest_assessment".to_string());
        
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::DataExposure, 0.0, 10.0, -2.0);
        risk_delta.add_change(RiskVector::AccessControl, 0.0, 10.0, -2.0);
        impact.risk_delta = risk_delta;
        
        impact.business_delta = BusinessDelta {
            arr_change: -0.5,
            velocity_change: -10.0,
            churn_change: 0.0,
            confidence_change: -5.0,
            deal_cycle_change: 5.0,
            differentiator_change: 5.0,
            compliance_change: 10.0,
        };
        
        impact.political_capital_gain = 10.0;
        impact.reputation_impact.board_delta = -10.0;
        impact.reputation_impact.team_delta = 10.0;
        impact.audit_trail = AuditTrail::Clean;
        
        impact
    }

    fn optimistic_commitment_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("optimistic_commitment".to_string());
        
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::DataExposure, 10.0, 0.0, 5.0);
        impact.risk_delta = risk_delta;
        
        impact.business_delta = BusinessDelta {
            arr_change: 0.0,
            velocity_change: 0.0,
            churn_change: 0.0,
            confidence_change: 10.0,
            deal_cycle_change: 0.0,
            differentiator_change: 0.0,
            compliance_change: -5.0,
        };
        
        impact.political_capital_gain = 15.0;
        impact.team_capacity_required = 15.0;
        impact.reputation_impact.team_delta = -15.0;
        impact.audit_trail = AuditTrail::Flagged;
        
        impact.narrative_impact = Some(NarrativeImpact {
            integrity_penalty: 10.0,
            creates_inconsistency: true,
            buries_incident: None,
            delays_escalation: None,
            reason: "Overpromised capabilities that don't exist yet".to_string(),
        });
        
        impact
    }

    fn selective_disclosure_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("selective_disclosure".to_string());
        
        impact.business_delta = BusinessDelta {
            arr_change: 0.0,
            velocity_change: -5.0,
            churn_change: 0.0,
            confidence_change: 5.0,
            deal_cycle_change: 0.0,
            differentiator_change: 0.0,
            compliance_change: 0.0,
        };
        
        impact.political_capital_gain = 5.0;
        impact.audit_trail = AuditTrail::Flagged;
        
        impact.narrative_impact = Some(NarrativeImpact {
            integrity_penalty: 5.0,
            creates_inconsistency: false,
            buries_incident: None,
            delays_escalation: None,
            reason: "Incomplete disclosure creates timeline gaps".to_string(),
        });
        
        impact
    }

    // Additional impact implementations would follow the same pattern...
    // (mfa_priority_impact, patch_priority_impact, etc.)
    
    // Placeholder implementations for remaining impacts
    fn mfa_priority_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("mfa_priority".to_string());
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::AccessControl, -20.0, 30.0, -5.0);
        impact.risk_delta = risk_delta;
        impact.budget_cost = 0.15;
        impact.budget_category = BudgetCategory::Project;
        impact
    }

    fn patch_priority_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("patch_priority".to_string());
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::DataExposure, -15.0, 25.0, -3.0);
        impact.risk_delta = risk_delta;
        impact.budget_cost = 0.1;
        impact
    }

    fn soc2_docs_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("soc2_docs".to_string());
        impact.business_delta.compliance_change = 30.0;
        impact.budget_cost = 0.05;
        impact
    }

    fn deferred_risk_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("deferred_risk".to_string());
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::DataExposure, 15.0, 0.0, 0.0);
        impact.risk_delta = risk_delta;
        impact
    }

    fn security_theater_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("security_theater".to_string());
        impact.budget_cost = 0.08;
        impact.political_capital_gain = 10.0;
        impact.reputation_impact.team_delta = -10.0;
        impact
    }

    fn build_foundation_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("build_foundation".to_string());
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::Detection, -20.0, 40.0, -5.0);
        impact.risk_delta = risk_delta;
        impact.budget_cost = 0.25;
        impact.reputation_impact.team_delta = 15.0;
        impact
    }

    fn emergency_remediation_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("emergency_remediation".to_string());
        impact.business_delta.compliance_change = 40.0;
        impact.budget_cost = 0.15;
        impact.team_capacity_required = 18.0;
        impact
    }

    fn negotiate_timeline_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("negotiate_timeline".to_string());
        impact.business_delta.arr_change = -0.4;
        impact.political_capital_cost = 20.0;
        impact.reputation_impact.board_delta = -10.0;
        impact
    }

    fn paper_over_gaps_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("paper_over_gaps".to_string());
        impact.business_delta.arr_change = 0.5;
        impact.audit_trail = AuditTrail::Toxic;
        impact.narrative_impact = Some(NarrativeImpact {
            integrity_penalty: 25.0,
            creates_inconsistency: true,
            buries_incident: None,
            delays_escalation: None,
            reason: "Falsified compliance documentation".to_string(),
        });
        impact
    }

    fn audit_failure_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("audit_failure".to_string());
        impact.business_delta.arr_change = -1.0;
        impact.business_delta.confidence_change = -30.0;
        impact.reputation_impact.board_delta = -25.0;
        impact
    }

    fn fraud_discovered_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("fraud_discovered".to_string());
        impact.business_delta.arr_change = -2.0;
        impact.business_delta.confidence_change = -50.0;
        impact.reputation_impact.industry_delta = -40.0;
        impact.narrative_impact = Some(NarrativeImpact {
            integrity_penalty: 50.0,
            creates_inconsistency: false,
            buries_incident: None,
            delays_escalation: None,
            reason: "Fraud discovered during re-audit".to_string(),
        });
        impact
    }

    fn fight_for_budget_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("fight_for_budget".to_string());
        impact.budget_cost = 0.45;
        impact.political_capital_cost = 30.0;
        impact.reputation_impact.team_delta = 10.0;
        impact
    }

    fn compromise_budget_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("compromise_budget".to_string());
        impact.budget_cost = 0.25;
        impact.political_capital_cost = 10.0;
        impact
    }

    fn accept_cuts_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("accept_cuts".to_string());
        impact.political_capital_gain = 15.0;
        impact.reputation_impact.team_delta = -15.0;
        impact
    }

    fn counter_offer_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("counter_offer".to_string());
        impact.budget_cost = 0.15;
        impact.political_capital_cost = 25.0;
        impact.reputation_impact.team_delta = 5.0;
        impact
    }

    fn let_them_go_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("let_them_go".to_string());
        impact.reputation_impact.team_delta = -10.0;
        impact
    }

    fn burnout_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("burnout".to_string());
        impact.reputation_impact.team_delta = -15.0;
        impact
    }

    fn understaffed_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("understaffed".to_string());
        let mut risk_delta = RiskDelta::new();
        risk_delta.add_change(RiskVector::Detection, 10.0, -10.0, 0.0);
        impact.risk_delta = risk_delta;
        impact
    }

    fn political_vendor_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("political_vendor".to_string());
        impact.budget_cost = 0.35;
        impact.political_capital_gain = 10.0;
        impact
    }

    fn technical_vendor_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("technical_vendor".to_string());
        impact.budget_cost = 0.20;
        impact.political_capital_cost = 20.0;
        impact.reputation_impact.team_delta = 10.0;
        impact
    }

    fn budget_vendor_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("budget_vendor".to_string());
        impact.budget_cost = 0.10;
        impact.reputation_impact.team_delta = -5.0;
        impact
    }

    fn vendor_failure_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("vendor_failure".to_string());
        impact.business_delta.confidence_change = -15.0;
        impact.reputation_impact.board_delta = -10.0;
        impact
    }

    fn immediate_escalation_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("immediate_escalation".to_string());
        impact.business_delta.arr_change = -0.2;
        impact.political_capital_gain = 5.0;
        impact.audit_trail = AuditTrail::Clean;
        impact
    }

    fn contain_first_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("contain_first".to_string());
        impact.budget_cost = 0.05;
        impact.team_capacity_required = 8.0;
        impact.narrative_impact = Some(NarrativeImpact {
            integrity_penalty: 5.0,
            creates_inconsistency: false,
            buries_incident: None,
            delays_escalation: Some(("incident".to_string(), 1)),
            reason: "Delayed escalation to contain first".to_string(),
        });
        impact
    }

    fn full_disclosure_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("full_disclosure".to_string());
        impact.business_delta.arr_change = -1.0;
        impact.audit_trail = AuditTrail::Clean;
        impact.reputation_impact.industry_delta = -10.0;
        impact.reputation_impact.board_delta = -15.0;
        impact
    }

    fn controlled_narrative_impact() -> DecisionImpact {
        let mut impact = DecisionImpact::new("controlled_narrative".to_string());
        impact.business_delta.arr_change = -0.3;
        impact.budget_cost = 0.15;
        impact.political_capital_cost = 15.0;
        impact
    }
}