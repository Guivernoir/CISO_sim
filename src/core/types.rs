use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;
use std::collections::HashMap;

/// Player information - now with baggage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub company_name: String,
    pub previous_role: String,
    pub reputation: Reputation,
}

impl Player {
    pub fn new(name: String, company_name: String, previous_role: String) -> Self {
        Self { 
            name, 
            company_name,
            previous_role,
            reputation: Reputation::new(),
        }
    }
}

/// Reputation - what people think when they hear your name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    pub industry_standing: f64,      // 0-100: Can you get another job after this?
    pub board_credibility: f64,       // 0-100: Do they believe you?
    pub team_morale: f64,             // 0-100: Will your team follow you into fire?
    pub vendor_relationships: f64,    // 0-100: Can you call in favors?
}

impl Reputation {
    pub fn new() -> Self {
        Self {
            industry_standing: 60.0,    // You're new, unknown
            board_credibility: 75.0,    // Fresh start bonus
            team_morale: 50.0,          // They miss the last guy
            vendor_relationships: 40.0, // You haven't built these yet
        }
    }
}

/// Opaque error types - never leak internal details
#[derive(Debug, Clone)]
pub enum GameError {
    StateCorruption,
    InvalidAction,
    SystemFailure,
    InsufficientBudget,
    InsufficientPoliticalCapital,
    TeamCapacityExceeded,
    ComplianceViolation,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::StateCorruption => write!(f, "Game state integrity check failed"),
            GameError::InvalidAction => write!(f, "Invalid action for current game state"),
            GameError::SystemFailure => write!(f, "System error occurred"),
            GameError::InsufficientBudget => write!(f, "Budget allocation failed"),
            GameError::InsufficientPoliticalCapital => write!(f, "Insufficient organizational capital"),
            GameError::TeamCapacityExceeded => write!(f, "Team bandwidth exceeded"),
            GameError::ComplianceViolation => write!(f, "Compliance framework violation"),
        }
    }
}

impl std::error::Error for GameError {}

/// Convert IO errors to opaque system failures - never leak paths or implementation details
impl From<std::io::Error> for GameError {
    fn from(_: std::io::Error) -> Self {
        GameError::SystemFailure
    }
}

pub type Result<T> = std::result::Result<T, GameError>;

/// Risk vectors - now with cascading failures and interdependencies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskVector {
    DataExposure,
    AccessControl,
    Detection,
    VendorRisk,
    InsiderThreat,
    SupplyChain,
    CloudMisconfiguration,
    APIAbuse,
}

/// Enhanced risk model - risks compound, decay, and cascade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLevel {
    pub vectors: HashMap<RiskVector, RiskMetric>,
    pub total_exposure: f64,
    pub risk_velocity: f64,  // How fast risk is growing
    pub cascade_multiplier: f64,  // Interdependency effects
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskMetric {
    pub current_level: f64,      // 0-100
    pub trend: f64,              // Rate of change
    pub time_to_critical: Option<u32>,  // Turns until this explodes
    pub mitigation_coverage: f64,  // 0-100: How well is this managed?
    pub last_incident: Option<u32>,  // Turn of last materialization
}

impl RiskMetric {
    pub fn new() -> Self {
        Self {
            current_level: 0.0,
            trend: 0.0,
            time_to_critical: None,
            mitigation_coverage: 0.0,
            last_incident: None,
        }
    }

    pub fn is_critical(&self) -> bool {
        self.current_level > 80.0 && self.mitigation_coverage < 30.0
    }

    pub fn is_degrading(&self) -> bool {
        self.trend > 5.0
    }
}

impl RiskLevel {
    pub fn new() -> Self {
        let mut vectors = HashMap::new();
        vectors.insert(RiskVector::DataExposure, RiskMetric::new());
        vectors.insert(RiskVector::AccessControl, RiskMetric::new());
        vectors.insert(RiskVector::Detection, RiskMetric::new());
        vectors.insert(RiskVector::VendorRisk, RiskMetric::new());
        vectors.insert(RiskVector::InsiderThreat, RiskMetric::new());
        vectors.insert(RiskVector::SupplyChain, RiskMetric::new());
        vectors.insert(RiskVector::CloudMisconfiguration, RiskMetric::new());
        vectors.insert(RiskVector::APIAbuse, RiskMetric::new());

        Self {
            vectors,
            total_exposure: 0.0,
            risk_velocity: 0.0,
            cascade_multiplier: 1.0,
        }
    }

    /// Apply natural risk decay (some things get better with time)
    pub fn apply_decay(&mut self, turn: u32) {
        for (vector, metric) in self.vectors.iter_mut() {
            // Controls degrade over time without maintenance
            if metric.last_incident.is_none() || turn - metric.last_incident.unwrap() > 3 {
                metric.mitigation_coverage *= 0.95; // 5% decay per turn
            }
            
            // Some risks naturally increase (tech debt, complexity)
            match vector {
                RiskVector::CloudMisconfiguration | RiskVector::APIAbuse => {
                    metric.current_level = (metric.current_level * 1.02).min(100.0);
                    metric.trend = 2.0;
                }
                _ => {}
            }
        }
    }

    /// Calculate cascading effects - one failure enables others
    pub fn calculate_cascade_effects(&mut self) {
        // Access control failure amplifies data exposure
        let access_level = self.vectors.get(&RiskVector::AccessControl)
            .map(|m| m.current_level).unwrap_or(0.0);
        
        if access_level > 60.0 {
            if let Some(data_metric) = self.vectors.get_mut(&RiskVector::DataExposure) {
                data_metric.current_level = (data_metric.current_level * 1.2).min(100.0);
            }
        }

        // Poor detection means everything is worse
        let detection_level = self.vectors.get(&RiskVector::Detection)
            .map(|m| m.mitigation_coverage).unwrap_or(0.0);
        
        if detection_level < 40.0 {
            self.cascade_multiplier = 1.5;
        } else {
            self.cascade_multiplier = 1.0;
        }

        // Vendor risk cascades to supply chain
        let vendor_level = self.vectors.get(&RiskVector::VendorRisk)
            .map(|m| m.current_level).unwrap_or(0.0);
        
        if vendor_level > 50.0 {
            if let Some(supply_metric) = self.vectors.get_mut(&RiskVector::SupplyChain) {
                supply_metric.current_level = (supply_metric.current_level * 1.15).min(100.0);
            }
        }

        self.total_exposure = self.vectors.values()
            .map(|m| m.current_level * (1.0 - m.mitigation_coverage / 100.0))
            .sum::<f64>() * self.cascade_multiplier;
    }

    pub fn apply_delta(&mut self, delta: &RiskDelta) {
        for (vector, change) in &delta.changes {
            if let Some(metric) = self.vectors.get_mut(vector) {
                metric.current_level = (metric.current_level + change.level_delta).max(0.0).min(100.0);
                metric.mitigation_coverage = (metric.mitigation_coverage + change.mitigation_delta).max(0.0).min(100.0);
                metric.trend = change.trend_delta;
                
                if change.level_delta > 0.0 && metric.current_level > 70.0 {
                    // Estimate turns to critical
                    let distance_to_critical = 100.0 - metric.current_level;
                    let rate = change.trend_delta.max(1.0);
                    metric.time_to_critical = Some((distance_to_critical / rate) as u32);
                }
            }
        }
    }
}

/// Risk deltas - now more granular with mitigation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDelta {
    pub changes: HashMap<RiskVector, RiskChange>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskChange {
    pub level_delta: f64,        // Change in risk level
    pub mitigation_delta: f64,   // Change in mitigation coverage
    pub trend_delta: f64,        // Change in risk velocity
}

impl RiskDelta {
    pub fn zero() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        Self::zero()
    }

    pub fn add_change(&mut self, vector: RiskVector, level: f64, mitigation: f64, trend: f64) {
        self.changes.insert(vector, RiskChange {
            level_delta: level,
            mitigation_delta: mitigation,
            trend_delta: trend,
        });
    }
}

/// Business metrics - the only thing that actually matters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub arr_millions: f64,
    pub roadmap_velocity_percent: f64,
    pub customer_churn_probability: f64,
    pub board_confidence_percent: f64,
    pub deal_cycle_days: f64,           // Security friction on sales
    pub security_as_differentiator: f64, // 0-100: Does security help sales?
    pub regulatory_compliance_score: f64, // 0-100: Multi-framework compliance
}

impl BusinessMetrics {
    pub fn new() -> Self {
        Self {
            arr_millions: 12.0,
            roadmap_velocity_percent: 100.0,
            customer_churn_probability: 5.0,
            board_confidence_percent: 70.0,
            deal_cycle_days: 45.0,
            security_as_differentiator: 30.0,
            regulatory_compliance_score: 40.0,
        }
    }

    pub fn apply_delta(&mut self, delta: &BusinessDelta) {
        self.arr_millions = (self.arr_millions + delta.arr_change).max(0.0);
        self.roadmap_velocity_percent = (self.roadmap_velocity_percent + delta.velocity_change).max(0.0);
        self.customer_churn_probability = (self.customer_churn_probability + delta.churn_change).max(0.0).min(100.0);
        self.board_confidence_percent = (self.board_confidence_percent + delta.confidence_change).max(0.0).min(100.0);
        self.deal_cycle_days = (self.deal_cycle_days + delta.deal_cycle_change).max(1.0);
        self.security_as_differentiator = (self.security_as_differentiator + delta.differentiator_change).max(0.0).min(100.0);
        self.regulatory_compliance_score = (self.regulatory_compliance_score + delta.compliance_change).max(0.0).min(100.0);
    }

    /// Calculate burn multiple - how efficiently are we growing?
    pub fn burn_multiple(&self, burn_rate: f64) -> f64 {
        if self.arr_millions == 0.0 { 
            return 99.0; // You're in trouble
        }
        burn_rate / (self.arr_millions / 12.0)
    }
}

/// Business impact deltas
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BusinessDelta {
    pub arr_change: f64,
    pub velocity_change: f64,
    pub churn_change: f64,
    pub confidence_change: f64,
    pub deal_cycle_change: f64,
    pub differentiator_change: f64,
    pub compliance_change: f64,
}

impl BusinessDelta {
    pub fn zero() -> Self {
        Self {
            arr_change: 0.0,
            velocity_change: 0.0,
            churn_change: 0.0,
            confidence_change: 0.0,
            deal_cycle_change: 0.0,
            differentiator_change: 0.0,
            compliance_change: 0.0,
        }
    }
}

/// Political capital - the hidden currency of corporate warfare
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PoliticalCapital {
    pub total: f64,              // 0-100
    pub ceo_favor: f64,          // 0-100
    pub cto_relationship: f64,   // 0-100
    pub cfo_trust: f64,          // 0-100
    pub earned_this_quarter: f64,
    pub spent_this_quarter: f64,
}

impl PoliticalCapital {
    pub fn new() -> Self {
        Self {
            total: 50.0,  // You start neutral
            ceo_favor: 60.0,
            cto_relationship: 40.0,  // CTOs often see security as blockers
            cfo_trust: 45.0,         // CFOs see you as cost center
            earned_this_quarter: 0.0,
            spent_this_quarter: 0.0,
        }
    }

    pub fn can_spend(&self, amount: f64) -> bool {
        self.total >= amount
    }

    pub fn spend(&mut self, amount: f64, target: Option<BoardMemberRole>) -> bool {
        if !self.can_spend(amount) {
            return false;
        }
        
        self.total -= amount;
        self.spent_this_quarter += amount;
        
        // Targeted spending affects relationships
        if let Some(role) = target {
            match role {
                BoardMemberRole::CEO => self.ceo_favor = (self.ceo_favor + amount * 0.5).min(100.0),
                BoardMemberRole::CTO => self.cto_relationship = (self.cto_relationship + amount * 0.5).min(100.0),
                BoardMemberRole::CFO => self.cfo_trust = (self.cfo_trust + amount * 0.5).min(100.0),
                _ => {}
            }
        }
        
        true
    }

    pub fn earn(&mut self, amount: f64, _source: String) {
        self.total = (self.total + amount).min(100.0);
        self.earned_this_quarter += amount;
    }

    pub fn quarterly_reset(&mut self) {
        self.earned_this_quarter = 0.0;
        self.spent_this_quarter = 0.0;
    }
}

/// Board members - they all want different things
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardMember {
    pub role: BoardMemberRole,
    pub name: String,
    pub personality: BoardPersonality,
    pub current_priority: BoardPriority,
    pub satisfaction: f64,  // 0-100
    pub influence: f64,     // 0-100: How much they sway decisions
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BoardMemberRole {
    CEO,
    CFO,
    CTO,
    COO,
    GeneralCounsel,
    BoardChair,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BoardPersonality {
    RiskAverse,        // Hates any bad news
    DataDriven,        // Wants metrics for everything
    PoliticallyShrewd, // Cares about optics
    TechnicallyMinded, // Understands the details
    BottomLineFocused, // Only cares about money
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BoardPriority {
    GrowthAtAllCosts,
    RiskMitigation,
    CostReduction,
    ComplianceFirst,
    CustomerTrust,
    IpoPreparation,
}

impl BoardMember {
    pub fn react_to_decision(&mut self, impact: &DecisionImpact) -> f64 {
        let mut satisfaction_delta = 0.0;
        
        match self.current_priority {
            BoardPriority::GrowthAtAllCosts => {
                satisfaction_delta += impact.business_delta.arr_change * 2.0;
                satisfaction_delta += impact.business_delta.velocity_change * 0.5;
            }
            BoardPriority::CostReduction => {
                satisfaction_delta -= impact.budget_cost * 5.0;
            }
            BoardPriority::RiskMitigation => {
                // Check if risk went down
                let risk_improvement: f64 = impact.risk_delta.changes.values()
                    .map(|c| -c.level_delta + c.mitigation_delta * 0.5)
                    .sum();
                satisfaction_delta += risk_improvement * 3.0;
            }
            BoardPriority::ComplianceFirst => {
                satisfaction_delta += impact.business_delta.compliance_change * 4.0;
            }
            BoardPriority::CustomerTrust => {
                satisfaction_delta -= impact.business_delta.churn_change * 3.0;
            }
            BoardPriority::IpoPreparation => {
                satisfaction_delta += impact.business_delta.compliance_change * 2.0;
                satisfaction_delta -= impact.business_delta.churn_change * 2.0;
            }
        }

        self.satisfaction = (self.satisfaction + satisfaction_delta).max(0.0).min(100.0);
        satisfaction_delta
    }
}

/// Team management - you can't do this alone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTeam {
    pub members: Vec<TeamMember>,
    pub total_capacity: f64,      // Story points per turn
    pub committed_capacity: f64,   // Already allocated
    pub morale: f64,              // 0-100
    pub attrition_risk: f64,      // 0-100: Probability of losing someone
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub role: SecurityRole,
    pub skill_level: f64,         // 0-100
    pub capacity: f64,            // Story points per turn
    pub burnout_level: f64,       // 0-100
    pub tenure_turns: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SecurityRole {
    SecurityEngineer,
    IncidentResponder,
    ComplianceAnalyst,
    SecurityArchitect,
    ThreatIntelligence,
    AppSec,
    CloudSecurity,
}

impl SecurityTeam {
    pub fn new() -> Self {
        // You inherit a skeleton crew
        let mut members = Vec::new();
        members.push(TeamMember {
            name: "Sarah Chen".to_string(),
            role: SecurityRole::SecurityEngineer,
            skill_level: 75.0,
            capacity: 10.0,
            burnout_level: 60.0,  // Already burned out from previous CISO
            tenure_turns: 8,
        });
        members.push(TeamMember {
            name: "Marcus Rodriguez".to_string(),
            role: SecurityRole::IncidentResponder,
            skill_level: 65.0,
            capacity: 8.0,
            burnout_level: 45.0,
            tenure_turns: 4,
        });

        Self {
            total_capacity: 18.0,
            committed_capacity: 12.0,  // Already firefighting
            morale: 45.0,
            attrition_risk: 35.0,
            members,
        }
    }

    pub fn available_capacity(&self) -> f64 {
        self.total_capacity - self.committed_capacity
    }

    pub fn allocate_capacity(&mut self, amount: f64) -> bool {
        if self.available_capacity() >= amount {
            self.committed_capacity += amount;
            true
        } else {
            false
        }
    }

    pub fn check_attrition(&mut self, _turn: u32) -> Vec<String> {
        let mut departed = Vec::new();
        
        self.members.retain(|member| {
            let leave_probability = (member.burnout_level + self.attrition_risk) / 200.0;
            let roll: f64 = rand::random();
            
            if roll < leave_probability {
                departed.push(member.name.clone());
                false
            } else {
                true
            }
        });

        departed
    }
}

/// Compliance frameworks - because one is never enough
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub frameworks: HashMap<ComplianceFramework, FrameworkStatus>,
    pub audit_schedule: Vec<ScheduledAudit>,
    pub open_findings: Vec<ComplianceFinding>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplianceFramework {
    SOC2,
    ISO27001,
    GDPR,
    HIPAA,
    PciDss,
    CCPA,
    StateBreachLaws,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkStatus {
    pub compliance_percent: f64,  // 0-100
    pub certification_date: Option<u32>,  // Turn when certified
    pub next_audit: u32,          // Turn of next audit
    pub control_gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledAudit {
    pub framework: ComplianceFramework,
    pub turn: u32,
    pub auditor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub framework: ComplianceFramework,
    pub severity: FindingSeverity,
    pub description: String,
    pub discovered_turn: u32,
    pub remediation_deadline: u32,
    pub status: FindingStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FindingStatus {
    Open,
    InProgress,
    Resolved,
    Accepted,  // Risk accepted by management
    Ignored,   // Well, that was quite the strategic decision, wasn't it?
}

impl ComplianceStatus {
    pub fn new() -> Self {
        let mut frameworks = HashMap::new();
        
        // You need SOC2 to sell to enterprises
        frameworks.insert(ComplianceFramework::SOC2, FrameworkStatus {
            compliance_percent: 40.0,  // Previous CISO let it slip
            certification_date: None,
            next_audit: 8,  // Audit in 8 turns
            control_gaps: vec![
                "Access reviews not performed".to_string(),
                "Change management process incomplete".to_string(),
                "Incident response plan not tested".to_string(),
            ],
        });

        Self {
            frameworks,
            audit_schedule: Vec::new(),
            open_findings: Vec::new(),
        }
    }
}

/// Narrative integrity - does your story survive discovery?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIntegrity {
    pub score: f64,
    pub inconsistencies: Vec<NarrativeInconsistency>,
    pub buried_incidents: Vec<BuriedIncident>,
    pub delayed_escalations: Vec<DelayedEscalation>,
    pub timeline_gaps: Vec<TimelineGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeInconsistency {
    pub turn: u32,
    pub description: String,
    pub severity: f64,
    pub evidence_trail: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuriedIncident {
    pub incident_id: String,
    pub actual_severity: IncidentSeverity,
    pub reported_severity: IncidentSeverity,
    pub turn_occurred: u32,
    pub turn_disclosed: Option<u32>,
    pub burial_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayedEscalation {
    pub incident_id: String,
    pub should_have_escalated_turn: u32,
    pub actually_escalated_turn: u32,
    pub delay_justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineGap {
    pub start_turn: u32,
    pub end_turn: u32,
    pub missing_context: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl NarrativeIntegrity {
    pub fn new() -> Self {
        Self {
            score: 100.0,
            inconsistencies: Vec::new(),
            buried_incidents: Vec::new(),
            delayed_escalations: Vec::new(),
            timeline_gaps: Vec::new(),
        }
    }

    pub fn record_inconsistency(&mut self, turn: u32, description: String, severity: f64) {
        self.score = (self.score - severity).max(0.0);
        self.inconsistencies.push(NarrativeInconsistency {
            turn,
            description,
            severity,
            evidence_trail: Vec::new(),
        });
    }

    pub fn bury_incident(&mut self, incident_id: String, actual: IncidentSeverity, 
                         reported: IncidentSeverity, turn: u32, method: String) {
        let severity_gap = self.severity_to_score(actual) - self.severity_to_score(reported);
        self.score = (self.score - severity_gap * 10.0).max(0.0);
        
        self.buried_incidents.push(BuriedIncident {
            incident_id,
            actual_severity: actual,
            reported_severity: reported,
            turn_occurred: turn,
            turn_disclosed: None,
            burial_method: method,
        });
    }

    pub fn delay_escalation(&mut self, incident_id: String, should_have: u32, 
                           actually: u32, justification: String) {
        let delay_turns = actually - should_have;
        self.score = (self.score - (delay_turns as f64 * 5.0)).max(0.0);
        
        self.delayed_escalations.push(DelayedEscalation {
            incident_id,
            should_have_escalated_turn: should_have,
            actually_escalated_turn: actually,
            delay_justification: justification,
        });
    }

    fn severity_to_score(&self, sev: IncidentSeverity) -> f64 {
        match sev {
            IncidentSeverity::Low => 1.0,
            IncidentSeverity::Medium => 2.0,
            IncidentSeverity::High => 3.0,
            IncidentSeverity::Critical => 4.0,
        }
    }

    /// Liability multiplier for lawsuits/fines
    pub fn get_multiplier(&self) -> f64 {
        if self.score > 85.0 {
            1.0  // Good faith
        } else if self.score > 50.0 {
            1.8  // Negligence
        } else {
            3.2  // Bad faith
        }
    }

    /// Are you going to prison?
    pub fn criminal_exposure(&self) -> bool {
        self.score < 30.0 && self.buried_incidents.len() > 2
    }
}

/// Budget - always insufficient
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Budget {
    pub total_annual: f64,
    pub spent: f64,
    pub committed: f64,
    pub headcount_budget: f64,
    pub tooling_budget: f64,
    pub project_budget: f64,
    pub emergency_reserve: f64,
}

impl Budget {
    pub fn new() -> Self {
        Self {
            total_annual: 2.5,
            spent: 0.0,
            committed: 0.8,
            headcount_budget: 1.2,
            tooling_budget: 0.6,
            project_budget: 0.4,
            emergency_reserve: 0.3,
        }
    }

    pub fn available(&self) -> f64 {
        self.total_annual - self.spent - self.committed
    }

    pub fn can_spend(&self, amount: f64, category: BudgetCategory) -> bool {
        let category_budget = match category {
            BudgetCategory::Headcount => self.headcount_budget,
            BudgetCategory::Tooling => self.tooling_budget,
            BudgetCategory::Project => self.project_budget,
            BudgetCategory::Emergency => self.emergency_reserve,
        };
        
        self.available() >= amount && category_budget >= amount
    }

    pub fn spend(&mut self, amount: f64, category: BudgetCategory) -> bool {
        if !self.can_spend(amount, category) {
            return false;
        }
        
        self.spent += amount;
        
        match category {
            BudgetCategory::Headcount => self.headcount_budget -= amount,
            BudgetCategory::Tooling => self.tooling_budget -= amount,
            BudgetCategory::Project => self.project_budget -= amount,
            BudgetCategory::Emergency => self.emergency_reserve -= amount,
        }
        
        true
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BudgetCategory {
    Headcount,
    Tooling,
    Project,
    Emergency,
}

/// Threat landscape - the world outside is hostile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatLandscape {
    pub current_threat_level: ThreatLevel,
    pub active_campaigns: Vec<ThreatCampaign>,
    pub industry_breaches: Vec<IndustryBreach>,
    pub exploit_availability: HashMap<String, ExploitStatus>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Baseline,
    Elevated,
    High,
    Severe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatCampaign {
    pub id: String,
    pub threat_actor: String,
    pub target_industry: String,
    pub active_since_turn: u32,
    pub techniques: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryBreach {
    pub company: String,
    pub turn: u32,
    pub impact: String,
    pub root_cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExploitStatus {
    PoCAvailable,
    ActivelyExploited,
    Weaponized,
}

impl ThreatLandscape {
    pub fn new() -> Self {
        Self {
            current_threat_level: ThreatLevel::Baseline,
            active_campaigns: Vec::new(),
            industry_breaches: Vec::new(),
            exploit_availability: HashMap::new(),
        }
    }

    pub fn evolve(&mut self, turn: u32) {
        // Threat level can change
        if turn % 4 == 0 {
            self.current_threat_level = match rand::random::<f64>() {
                x if x < 0.5 => ThreatLevel::Baseline,
                x if x < 0.8 => ThreatLevel::Elevated,
                x if x < 0.95 => ThreatLevel::High,
                _ => ThreatLevel::Severe,
            };
        }
    }
}

/// Audit trail quality - do you want discovery to find this?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AuditTrail {
    Clean,
    Flagged,
    Toxic,
}

/// Session token - zeroized on drop
#[derive(Zeroize, Clone)]
#[zeroize(drop)]
pub struct SessionToken {
    pub data: [u8; 32],
}

impl SessionToken {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut data = [0u8; 32];
        rng.fill(&mut data);
        Self { data }
    }
}

/// Decision impact - now tracks everything
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionImpact {
    pub decision_id: String,
    pub risk_delta: RiskDelta,
    pub business_delta: BusinessDelta,
    pub budget_cost: f64,
    pub budget_category: BudgetCategory,
    pub political_capital_cost: f64,
    pub political_capital_gain: f64,
    pub team_capacity_required: f64,
    pub reputation_impact: ReputationDelta,
    pub compliance_impact: ComplianceImpact,
    pub narrative_impact: Option<NarrativeImpact>,
    pub audit_trail: AuditTrail,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReputationDelta {
    pub industry_delta: f64,
    pub board_delta: f64,
    pub team_delta: f64,
    pub vendor_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceImpact {
    pub framework_progress: HashMap<ComplianceFramework, f64>,
    pub new_findings: Vec<ComplianceFinding>,
    pub resolved_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeImpact {
    pub integrity_penalty: f64,
    pub creates_inconsistency: bool,
    pub buries_incident: Option<(String, IncidentSeverity, IncidentSeverity)>,
    pub delays_escalation: Option<(String, u32)>,
    pub reason: String,
}

impl DecisionImpact {
    pub fn new(id: String) -> Self {
        Self {
            decision_id: id,
            risk_delta: RiskDelta::zero(),
            business_delta: BusinessDelta::zero(),
            budget_cost: 0.0,
            budget_category: BudgetCategory::Project,
            political_capital_cost: 0.0,
            political_capital_gain: 0.0,
            team_capacity_required: 0.0,
            reputation_impact: ReputationDelta {
                industry_delta: 0.0,
                board_delta: 0.0,
                team_delta: 0.0,
                vendor_delta: 0.0,
            },
            compliance_impact: ComplianceImpact {
                framework_progress: HashMap::new(),
                new_findings: Vec::new(),
                resolved_findings: Vec::new(),
            },
            narrative_impact: None,
            audit_trail: AuditTrail::Clean,
        }
    }
}