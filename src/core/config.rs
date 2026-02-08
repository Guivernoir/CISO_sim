use crate::core::types::*;
use crate::core::decisions::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DecisionConfig {
    pub decision: DecisionMeta,
    choices: Vec<ChoiceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DecisionMeta {
    id: String,
    turn: u32,
    title: String,
    is_board_pressure: bool,
    is_time_sensitive: Option<bool>,
    decision_category: Option<String>,
    context: String,
}

#[derive(Debug, Deserialize)]
struct ChoiceConfig {
    id: String,
    label: String,
    description: String,
    impact_preview: ImpactPreviewConfig,
    impact: ImpactConfig,
    prerequisites: Option<PrerequisitesConfig>,
}

#[derive(Debug, Deserialize)]
struct PrerequisitesConfig {
    min_budget: Option<f64>,
    min_political_capital: Option<f64>,
    min_team_capacity: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ImpactPreviewConfig {
    estimated_arr_change: f64,
    budget_cost: f64,
    timeline_weeks: Option<u32>,
    political_note: Option<String>,
    risk_indicator: Option<String>,
    team_impact: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImpactConfig {
    risk_delta: RiskDeltaConfig,
    business_delta: BusinessDeltaConfig,
    budget_cost: f64,
    budget_category: Option<String>,
    political_capital_cost: Option<f64>,
    political_capital_gain: Option<f64>,
    team_capacity_required: Option<f64>,
    audit_trail: String,
    reputation_impact: Option<ReputationDeltaConfig>,
    narrative_impact: Option<NarrativeImpactConfig>,
}

#[derive(Debug, Deserialize)]
struct RiskDeltaConfig {
    data_exposure: Option<f64>,
    access_control: Option<f64>,
    detection: Option<f64>,
    vendor_risk: Option<f64>,
    insider_threat: Option<f64>,
    _supply_chain: Option<f64>,
    _cloud_misconfiguration: Option<f64>,
    _api_abuse: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct BusinessDeltaConfig {
    arr_change: f64,
    velocity_change: f64,
    churn_change: f64,
    confidence_change: f64,
    deal_cycle_change: Option<f64>,
    differentiator_change: Option<f64>,
    compliance_change: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ReputationDeltaConfig {
    industry_delta: Option<f64>,
    board_delta: Option<f64>,
    team_delta: Option<f64>,
    vendor_delta: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct NarrativeImpactConfig {
    integrity_penalty: f64,
    creates_inconsistency: bool,
    reason: String,
}

pub struct DecisionLoader {
    pub decisions: HashMap<u32, Decision>,
}

impl DecisionLoader {
    pub fn new() -> Result<Self> {
        let mut _decisions: HashMap<u32, Decision> = HashMap::new();
        
        // Load all decision files from data/decisions/
        let data_dir = Path::new("data/decisions");
        
        if !data_dir.exists() {
            // Try relative to executable
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let data_dir = exe_dir.join("data/decisions");
                    
                    if data_dir.exists() {
                        return Self::load_from_dir(&data_dir);
                    }
                }
            }
            
            // Fall back to current directory
            return Self::load_from_dir(Path::new("data/decisions"));
        }
        
        Self::load_from_dir(data_dir)
    }
    
    fn load_from_dir(dir: &Path) -> Result<Self> {
        let mut decisions: HashMap<u32, Decision> = HashMap::new();
        
        if !dir.exists() {
            // If no data directory, return empty loader (will fall back to DecisionFactory)
            return Ok(Self { decisions });
        }
        
        let entries = fs::read_dir(dir).map_err(|_| GameError::SystemFailure)?;
        
        for entry in entries {
            let entry = entry.map_err(|_| GameError::SystemFailure)?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path)
                    .map_err(|_| GameError::SystemFailure)?;
                
                let config: DecisionConfig = toml::from_str(&content)
                    .map_err(|_| GameError::StateCorruption)?;
                
                let decision = Self::convert_decision(config)?;
                decisions.insert(decision.turn, decision);
            }
        }
        
        Ok(Self { decisions })
    }
    
    fn convert_decision(config: DecisionConfig) -> Result<Decision> {
        let choices = config.choices.into_iter()
            .map(|c| Self::convert_choice(c))
            .collect();
        
        let decision_category = config.decision.decision_category
            .as_ref()
            .and_then(|cat| match cat.as_str() {
                "StrategicDirection" => Some(DecisionCategory::StrategicDirection),
                "IncidentResponse" => Some(DecisionCategory::IncidentResponse),
                "BudgetAllocation" => Some(DecisionCategory::BudgetAllocation),
                "ComplianceApproach" => Some(DecisionCategory::ComplianceApproach),
                "TeamManagement" => Some(DecisionCategory::TeamManagement),
                "VendorSelection" => Some(DecisionCategory::VendorSelection),
                "RiskAcceptance" => Some(DecisionCategory::RiskAcceptance),
                "PoliticalNavigation" => Some(DecisionCategory::PoliticalNavigation),
                _ => None,
            })
            .unwrap_or(DecisionCategory::StrategicDirection);
        
        Ok(Decision {
            id: config.decision.id,
            turn: config.decision.turn,
            title: config.decision.title,
            context: config.decision.context,
            choices,
            is_board_pressure: config.decision.is_board_pressure,
            is_time_sensitive: config.decision.is_time_sensitive.unwrap_or(false),
            decision_category,
            prerequisites: Vec::new(),
        })
    }
    
    fn convert_choice(config: ChoiceConfig) -> Choice {
        let risk_indicator = config.impact_preview.risk_indicator
            .as_ref()
            .and_then(|ri| match ri.as_str() {
                "Reduces" => Some(RiskIndicator::Reduces),
                "Neutral" => Some(RiskIndicator::Neutral),
                "Increases" => Some(RiskIndicator::Increases),
                "Significant" => Some(RiskIndicator::Significant),
                _ => None,
            })
            .unwrap_or(RiskIndicator::Neutral);

        let prerequisites = if let Some(prereq_config) = &config.prerequisites {
            ChoicePrerequisites {
                min_budget: prereq_config.min_budget.unwrap_or(0.0),
                min_political_capital: prereq_config.min_political_capital.unwrap_or(0.0),
                min_team_capacity: prereq_config.min_team_capacity.unwrap_or(0.0),
                required_compliance: Vec::new(),
                blocked_by: Vec::new(),
            }
        } else {
            ChoicePrerequisites::default()
        };

        Choice {
            id: config.id,
            label: config.label,
            description: config.description,
            impact_preview: ImpactPreview {
                estimated_arr_change: config.impact_preview.estimated_arr_change,
                budget_cost: config.impact_preview.budget_cost,
                timeline_weeks: config.impact_preview.timeline_weeks,
                political_note: config.impact_preview.political_note,
                risk_indicator,
                compliance_impact: ComplianceImpact {
                    framework_progress: HashMap::new(),
                    new_findings: Vec::new(),
                    resolved_findings: Vec::new(),
                },
                team_impact: config.impact_preview.team_impact.unwrap_or_default(),
            },
            impact_data: Some(Self::convert_impact(config.impact)),
            prerequisites,
            consequences: Vec::new(),
        }
    }
    
    fn convert_impact(config: ImpactConfig) -> DecisionImpact {
        let mut impact = DecisionImpact::new("from_config".to_string());
        
        // Convert risk delta
        let mut risk_delta = RiskDelta::new();
        
        if let Some(val) = config.risk_delta.data_exposure {
            risk_delta.add_change(RiskVector::DataExposure, val, 0.0, 0.0);
        }
        if let Some(val) = config.risk_delta.access_control {
            risk_delta.add_change(RiskVector::AccessControl, val, 0.0, 0.0);
        }
        if let Some(val) = config.risk_delta.detection {
            risk_delta.add_change(RiskVector::Detection, val, 0.0, 0.0);
        }
        if let Some(val) = config.risk_delta.vendor_risk {
            risk_delta.add_change(RiskVector::VendorRisk, val, 0.0, 0.0);
        }
        if let Some(val) = config.risk_delta.insider_threat {
            risk_delta.add_change(RiskVector::InsiderThreat, val, 0.0, 0.0);
        }
        
        impact.risk_delta = risk_delta;
        
        impact.business_delta = BusinessDelta {
            arr_change: config.business_delta.arr_change,
            velocity_change: config.business_delta.velocity_change,
            churn_change: config.business_delta.churn_change,
            confidence_change: config.business_delta.confidence_change,
            deal_cycle_change: config.business_delta.deal_cycle_change.unwrap_or(0.0),
            differentiator_change: config.business_delta.differentiator_change.unwrap_or(0.0),
            compliance_change: config.business_delta.compliance_change.unwrap_or(0.0),
        };
        
        impact.budget_cost = config.budget_cost;
        
        impact.budget_category = config.budget_category
            .as_ref()
            .and_then(|cat| match cat.as_str() {
                "Headcount" => Some(BudgetCategory::Headcount),
                "Tooling" => Some(BudgetCategory::Tooling),
                "Project" => Some(BudgetCategory::Project),
                "Emergency" => Some(BudgetCategory::Emergency),
                _ => None,
            })
            .unwrap_or(BudgetCategory::Project);
        
        impact.political_capital_cost = config.political_capital_cost.unwrap_or(0.0);
        impact.political_capital_gain = config.political_capital_gain.unwrap_or(0.0);
        impact.team_capacity_required = config.team_capacity_required.unwrap_or(0.0);
        
        impact.audit_trail = match config.audit_trail.as_str() {
            "Clean" => AuditTrail::Clean,
            "Flagged" => AuditTrail::Flagged,
            "Toxic" => AuditTrail::Toxic,
            _ => AuditTrail::Clean,
        };
        
        if let Some(rep_config) = config.reputation_impact {
            impact.reputation_impact = ReputationDelta {
                industry_delta: rep_config.industry_delta.unwrap_or(0.0),
                board_delta: rep_config.board_delta.unwrap_or(0.0),
                team_delta: rep_config.team_delta.unwrap_or(0.0),
                vendor_delta: rep_config.vendor_delta.unwrap_or(0.0),
            };
        }
        
        if let Some(narrative_config) = config.narrative_impact {
            impact.narrative_impact = Some(NarrativeImpact {
                integrity_penalty: narrative_config.integrity_penalty,
                creates_inconsistency: narrative_config.creates_inconsistency,
                buries_incident: None,
                delays_escalation: None,
                reason: narrative_config.reason,
            });
        }
        
        impact
    }
    
    pub fn get_decision(&self, turn: u32) -> Option<&Decision> {
        self.decisions.get(&turn)
    }
}