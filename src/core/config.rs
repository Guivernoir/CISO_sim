use crate::core::types::*;
use crate::core::decisions::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct TomlRoot {
    pub decision: Vec<DecisionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DecisionConfig {
    pub turn: u32,
    pub title: String,
    pub context: String,
    #[serde(default)]
    pub is_board_pressure: bool,
    #[serde(default)]
    pub is_time_sensitive: bool,
    #[serde(default)]
    pub decision_category: Option<String>,
    pub choice: Vec<ChoiceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceConfig {
    pub id: String,
    pub label: String,
    pub description: String,
    pub impact_preview: ImpactPreviewConfig,
    pub impact: ImpactConfigWrapper,
    #[serde(default)]
    pub prerequisites: Option<PrerequisitesConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PrerequisitesConfig {
    pub min_budget: Option<f64>,
    pub min_political_capital: Option<f64>,
    pub min_team_capacity: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ImpactPreviewConfig {
    pub estimated_arr_change: f64,
    pub budget_cost: f64,
    #[serde(default)]
    pub timeline_weeks: Option<u32>,
    #[serde(default)]
    pub political_note: Option<String>,
    #[serde(default)]
    pub risk_indicator: Option<String>,
    #[serde(default)]
    pub team_impact: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImpactConfigWrapper {
    #[serde(default)]
    pub risk_delta: Option<RiskDeltaConfig>,
    #[serde(default)]
    pub business_delta: Option<BusinessDeltaConfig>,
    #[serde(default)]
    pub audit_trail: Option<String>,
    #[serde(default)]
    pub budget_impact: Option<f64>,
    #[serde(default)]
    pub budget_category: Option<String>,
    #[serde(default)]
    pub political_capital_cost: Option<f64>,
    #[serde(default)]
    pub political_capital_gain: Option<f64>,
    #[serde(default)]
    pub team_capacity_required: Option<f64>,
    #[serde(default)]
    pub reputation_impact: Option<ReputationDeltaConfig>,
    #[serde(default)]
    pub narrative_impact: Option<NarrativeImpactConfig>,
}

#[derive(Debug, Deserialize)]
pub struct RiskDeltaConfig {
    #[serde(default)]
    pub changes: Option<HashMap<String, RiskChangeConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct RiskChangeConfig {
    pub level_delta: f64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mitigation_delta: Option<f64>,
    #[serde(default)]
    pub trend_delta: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct BusinessDeltaConfig {
    pub arr_change: f64,
    pub velocity_change: f64,
    pub churn_change: f64,
    pub confidence_change: f64,
    #[serde(default)]
    pub deal_cycle_change: Option<f64>,
    #[serde(default)]
    pub differentiator_change: Option<f64>,
    #[serde(default)]
    pub compliance_change: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ReputationDeltaConfig {
    pub industry_delta: Option<f64>,
    pub board_delta: Option<f64>,
    pub team_delta: Option<f64>,
    pub vendor_delta: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct NarrativeImpactConfig {
    pub integrity_penalty: f64,
    pub creates_inconsistency: bool,
    pub reason: String,
}

pub struct DecisionLoader {
    pub decisions: HashMap<u32, Decision>,
}

impl DecisionLoader {
    pub fn new() -> Result<Self> {
        // Try to load from data/decisions directory
        let data_dir = Path::new("data/decisions");
        
        if data_dir.exists() {
            return Self::load_from_dir(data_dir);
        }
        
        // Try relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let alt_dir = exe_dir.join("data/decisions");
                if alt_dir.exists() {
                    return Self::load_from_dir(&alt_dir);
                }
            }
        }
        
        // Return empty loader (will fall back to DecisionFactory)
        Ok(Self { 
            decisions: HashMap::new() 
        })
    }
    
    fn load_from_dir(dir: &Path) -> Result<Self> {
        let mut decisions: HashMap<u32, Decision> = HashMap::new();
        
        let entries = fs::read_dir(dir).map_err(|_| GameError::SystemFailure)?;
        
        for entry in entries {
            let entry = entry.map_err(|_| GameError::SystemFailure)?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path)
                    .map_err(|_| GameError::SystemFailure)?;
                
                let root: TomlRoot = toml::from_str(&content)
                    .map_err(|_| GameError::StateCorruption)?;
                
                for decision_config in root.decision {
                    let decision = Self::convert_decision(decision_config)?;
                    decisions.insert(decision.turn, decision);
                }
            }
        }
        
        Ok(Self { decisions })
    }
    
    fn convert_decision(config: DecisionConfig) -> Result<Decision> {
        let choices = config.choice.into_iter()
            .map(Self::convert_choice)
            .collect();
        
        let decision_category = config.decision_category
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
            id: format!("turn_{}", config.turn),
            turn: config.turn,
            title: config.title,
            context: config.context,
            choices,
            is_board_pressure: config.is_board_pressure,
            is_time_sensitive: config.is_time_sensitive,
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

        let prerequisites = config.prerequisites
            .map(|prereq_config| ChoicePrerequisites {
                min_budget: prereq_config.min_budget.unwrap_or(0.0),
                min_political_capital: prereq_config.min_political_capital.unwrap_or(0.0),
                min_team_capacity: prereq_config.min_team_capacity.unwrap_or(0.0),
                required_compliance: Vec::new(),
                blocked_by: Vec::new(),
            })
            .unwrap_or_default();

        Choice {
            id: config.id.clone(),
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
            impact_data: Some(Self::convert_impact(&config.id, config.impact)),
            prerequisites,
            consequences: Vec::new(),
        }
    }
    
    fn convert_impact(choice_id: &str, config: ImpactConfigWrapper) -> DecisionImpact {
        let mut impact = DecisionImpact::new(choice_id.to_string());
        
        // Convert risk delta
        if let Some(risk_delta_config) = config.risk_delta {
            let mut risk_delta = RiskDelta::new();
            
            if let Some(changes) = risk_delta_config.changes {
                for (vector_name, change) in changes {
                    let vector = match vector_name.as_str() {
                        "DataExposure" => RiskVector::DataExposure,
                        "AccessControl" => RiskVector::AccessControl,
                        "Detection" => RiskVector::Detection,
                        "VendorRisk" => RiskVector::VendorRisk,
                        "InsiderThreat" => RiskVector::InsiderThreat,
                        "SupplyChain" => RiskVector::SupplyChain,
                        "CloudMisconfiguration" => RiskVector::CloudMisconfiguration,
                        "APIAbuse" => RiskVector::APIAbuse,
                        _ => continue,
                    };
                    
                    risk_delta.add_change(
                        vector,
                        change.level_delta,
                        change.mitigation_delta.unwrap_or(0.0),
                        change.trend_delta.unwrap_or(0.0)
                    );
                }
            }
            
            impact.risk_delta = risk_delta;
        }
        
        // Convert business delta
        if let Some(business_config) = config.business_delta {
            impact.business_delta = BusinessDelta {
                arr_change: business_config.arr_change,
                velocity_change: business_config.velocity_change,
                churn_change: business_config.churn_change,
                confidence_change: business_config.confidence_change,
                deal_cycle_change: business_config.deal_cycle_change.unwrap_or(0.0),
                differentiator_change: business_config.differentiator_change.unwrap_or(0.0),
                compliance_change: business_config.compliance_change.unwrap_or(0.0),
            };
        }
        
        // Budget cost - use budget_impact field from TOML, make it positive
        impact.budget_cost = config.budget_impact.map(|v| v.abs()).unwrap_or(0.0);
        
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
        
        impact.audit_trail = config.audit_trail
            .as_ref()
            .and_then(|trail| match trail.as_str() {
                "Clean" => Some(AuditTrail::Clean),
                "Flagged" => Some(AuditTrail::Flagged),
                "Toxic" => Some(AuditTrail::Toxic),
                _ => None,
            })
            .unwrap_or(AuditTrail::Clean);
        
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