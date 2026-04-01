use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillDomain {
    Building,
    Clothing,
    Scripting,
    Landscaping,
    Vehicles,
    Media,
    Navigation,
    Estate,
    Economy,
    Social,
    Animation,
    Inventory,
    NpcManagement,
    Tutorial,
}

impl SkillDomain {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Building => "building",
            Self::Clothing => "clothing",
            Self::Scripting => "scripting",
            Self::Landscaping => "landscaping",
            Self::Vehicles => "vehicles",
            Self::Media => "media",
            Self::Navigation => "navigation",
            Self::Estate => "estate",
            Self::Economy => "economy",
            Self::Social => "social",
            Self::Animation => "animation",
            Self::Inventory => "inventory",
            Self::NpcManagement => "npc_management",
            Self::Tutorial => "tutorial",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Building => "Building",
            Self::Clothing => "Clothing",
            Self::Scripting => "Scripting",
            Self::Landscaping => "Landscaping",
            Self::Vehicles => "Vehicles",
            Self::Media => "Media",
            Self::Navigation => "Navigation",
            Self::Estate => "Estate",
            Self::Economy => "Economy",
            Self::Social => "Social",
            Self::Animation => "Animation",
            Self::Inventory => "Inventory",
            Self::NpcManagement => "NPC Management",
            Self::Tutorial => "Tutorial",
        }
    }

    pub fn all() -> &'static [SkillDomain] {
        &[
            Self::Building, Self::Clothing, Self::Scripting,
            Self::Landscaping, Self::Vehicles, Self::Media,
            Self::Navigation, Self::Estate, Self::Economy,
            Self::Social, Self::Animation, Self::Inventory,
            Self::NpcManagement, Self::Tutorial,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SkillMaturity {
    L0Seed = 0,
    L1Defined = 1,
    L2Stubbed = 2,
    L3Functional = 3,
    L4Robust = 4,
    L5Integrated = 5,
    L6Verified = 6,
    L7Production = 7,
}

impl SkillMaturity {
    pub fn level(&self) -> u8 {
        *self as u8
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::L0Seed => "Seed",
            Self::L1Defined => "Defined",
            Self::L2Stubbed => "Stubbed",
            Self::L3Functional => "Functional",
            Self::L4Robust => "Robust",
            Self::L5Integrated => "Integrated",
            Self::L6Verified => "Verified",
            Self::L7Production => "Production",
        }
    }

    pub fn score_weight(&self) -> u32 {
        match self {
            Self::L0Seed => 0,
            Self::L1Defined => 5,
            Self::L2Stubbed => 15,
            Self::L3Functional => 30,
            Self::L4Robust => 50,
            Self::L5Integrated => 70,
            Self::L6Verified => 85,
            Self::L7Production => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    F32,
    U32,
    Bool,
    String,
    Uuid,
    Vec3,
    Vec4,
    StringArray,
    U32Array,
    F32Map,
    StringMap,
    Enum(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReturnType {
    Success,
    LocalId,
    LocalIds,
    ObjectData,
    TextResponse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SkillExample {
    pub description: &'static str,
    pub input: &'static str,
    pub output: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDef {
    pub name: &'static str,
    pub param_type: ParamType,
    pub required: bool,
    pub default_value: Option<&'static str>,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct SkillDef {
    pub id: &'static str,
    pub domain: SkillDomain,
    pub display_name: &'static str,
    pub description: &'static str,
    pub params: &'static [ParamDef],
    pub returns: ReturnType,
    pub requires_region: bool,
    pub requires_agent: bool,
    pub requires_admin: bool,
    pub maturity: SkillMaturity,
    pub phase: &'static str,
    pub tags: &'static [&'static str],
    pub examples: &'static [SkillExample],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub domain: String,
    pub display_name: String,
    pub description: String,
    pub maturity: u8,
    pub maturity_label: String,
    pub phase: String,
    pub tags: Vec<String>,
    pub requires_region: bool,
    pub requires_agent: bool,
    pub requires_admin: bool,
    pub params: Vec<ParamInfo>,
    pub examples: Vec<SkillExampleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExampleInfo {
    pub description: String,
    pub input: String,
    pub output: String,
}

impl SkillDef {
    pub fn to_info(&self) -> SkillInfo {
        SkillInfo {
            id: self.id.to_string(),
            domain: self.domain.id().to_string(),
            display_name: self.display_name.to_string(),
            description: self.description.to_string(),
            maturity: self.maturity.level(),
            maturity_label: self.maturity.label().to_string(),
            phase: self.phase.to_string(),
            tags: self.tags.iter().map(|t| t.to_string()).collect(),
            requires_region: self.requires_region,
            requires_agent: self.requires_agent,
            requires_admin: self.requires_admin,
            params: self.params.iter().map(|p| ParamInfo {
                name: p.name.to_string(),
                param_type: format!("{:?}", p.param_type),
                required: p.required,
                default_value: p.default_value.map(|v| v.to_string()),
                description: p.description.to_string(),
            }).collect(),
            examples: self.examples.iter().map(|e| SkillExampleInfo {
                description: e.description.to_string(),
                input: e.input.to_string(),
                output: e.output.to_string(),
            }).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDashboard {
    pub domain: String,
    pub display_name: String,
    pub total_skills: usize,
    pub by_level: [usize; 8],
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryDashboard {
    pub total_skills: usize,
    pub total_domains: usize,
    pub domains: Vec<DomainDashboard>,
    pub overall_score: u32,
}

pub struct SkillRegistry {
    skills: HashMap<&'static str, &'static SkillDef>,
    by_domain: HashMap<SkillDomain, Vec<&'static str>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            skills: HashMap::new(),
            by_domain: HashMap::new(),
        };
        super::skill_defs::register_all(&mut registry);
        registry
    }

    pub fn register(&mut self, skill: &'static SkillDef) {
        self.skills.insert(skill.id, skill);
        self.by_domain
            .entry(skill.domain)
            .or_default()
            .push(skill.id);
    }

    pub fn get(&self, skill_id: &str) -> Option<&'static SkillDef> {
        self.skills.get(skill_id).copied()
    }

    pub fn find(&self, domain: SkillDomain, skill_id: &str) -> Option<&'static SkillDef> {
        let def = self.skills.get(skill_id).copied()?;
        if def.domain == domain { Some(def) } else { None }
    }

    pub fn list_domain(&self, domain: SkillDomain) -> Vec<&'static SkillDef> {
        self.by_domain
            .get(&domain)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id).copied()).collect())
            .unwrap_or_default()
    }

    pub fn list_all(&self) -> Vec<&'static SkillDef> {
        self.skills.values().copied().collect()
    }

    pub fn search(&self, query: &str) -> Vec<&'static SkillDef> {
        let q = query.to_lowercase();
        self.skills.values()
            .copied()
            .filter(|s| {
                s.id.contains(&q)
                    || s.display_name.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
                    || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    pub fn domain_dashboard(&self, domain: SkillDomain) -> DomainDashboard {
        let skills = self.list_domain(domain);
        let mut by_level = [0usize; 8];
        let mut total_score: u32 = 0;
        for skill in &skills {
            by_level[skill.maturity.level() as usize] += 1;
            total_score += skill.maturity.score_weight();
        }
        let score = if skills.is_empty() { 0 } else { total_score / skills.len() as u32 };
        DomainDashboard {
            domain: domain.id().to_string(),
            display_name: domain.display_name().to_string(),
            total_skills: skills.len(),
            by_level,
            score,
        }
    }

    pub fn generate_prompt_catalog(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("\n== SKILL ENGINE CATALOG (auto-generated) ==\n\n");
        out.push_str(&format!("You have access to {} skills across {} domains via the Skill Engine.\n", self.count(), SkillDomain::all().len()));
        out.push_str("NEW DOMAINS beyond building/clothing/scripting/landscaping/vehicles/media:\n");
        out.push_str("  Navigation, Estate, Economy, Social, Animation, Inventory, NPC Management, Tutorial\n\n");
        out.push_str("For NEW domain skills, output JSON using osInvokeSkill format:\n");
        out.push_str(r#"{"actions": [{"os_invoke_skill": {"domain": "DOMAIN", "skill_id": "SKILL_ID", "params": {...}}}], "say": "..."}"#);
        out.push_str("\n\nFor EXISTING domain skills (building, clothing, scripting, landscaping, vehicles, media), continue using the original action format documented above.\n\n");

        for domain in SkillDomain::all() {
            let skills = self.list_domain(*domain);
            if skills.is_empty() { continue; }
            let dash = self.domain_dashboard(*domain);
            out.push_str(&format!("── {} ({} skills, {}% maturity) ──\n", domain.display_name(), skills.len(), dash.score));
            for skill in &skills {
                let params_str: String = skill.params.iter()
                    .map(|p| {
                        if p.required { p.name.to_string() } else { format!("[{}]", p.name) }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("  {} (L{}) — {} | params: {}\n",
                    skill.id, skill.maturity.level(), skill.description, params_str));
            }
            out.push('\n');
        }
        out
    }

    pub fn dashboard(&self) -> RegistryDashboard {
        let domains: Vec<DomainDashboard> = SkillDomain::all()
            .iter()
            .map(|d| self.domain_dashboard(*d))
            .collect();
        let total_skills: usize = domains.iter().map(|d| d.total_skills).sum();
        let total_score: u32 = if total_skills == 0 { 0 } else {
            domains.iter().map(|d| d.score as u64 * d.total_skills as u64).sum::<u64>() as u32 / total_skills as u32
        };
        RegistryDashboard {
            total_skills,
            total_domains: domains.len(),
            domains,
            overall_score: total_score,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillResult {
    Ok { message: String },
    OkWithId { message: String, local_id: u32 },
    OkWithIds { message: String, local_ids: Vec<u32> },
    OkWithData { message: String, data: serde_json::Value },
    NotImplemented { skill_id: String },
    Error { message: String },
    Forbidden { message: String },
}

impl SkillResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self::Ok { message: message.into() }
    }

    pub fn ok_with_id(message: impl Into<String>, local_id: u32) -> Self {
        Self::OkWithId { message: message.into(), local_id }
    }

    pub fn ok_with_ids(message: impl Into<String>, local_ids: Vec<u32>) -> Self {
        Self::OkWithIds { message: message.into(), local_ids }
    }

    pub fn ok_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self::OkWithData { message: message.into(), data }
    }

    pub fn not_implemented(skill_id: impl Into<String>) -> Self {
        Self::NotImplemented { skill_id: skill_id.into() }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error { message: message.into() }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden { message: message.into() }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok { .. } | Self::OkWithId { .. } | Self::OkWithIds { .. } | Self::OkWithData { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Forbidden { .. } | Self::NotImplemented { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_domain_all() {
        assert_eq!(SkillDomain::all().len(), 14);
    }

    #[test]
    fn test_skill_domain_ids_unique() {
        let ids: Vec<&str> = SkillDomain::all().iter().map(|d| d.id()).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn test_maturity_ordering() {
        assert!(SkillMaturity::L0Seed < SkillMaturity::L7Production);
        assert!(SkillMaturity::L3Functional < SkillMaturity::L4Robust);
    }

    #[test]
    fn test_maturity_score_weights() {
        assert_eq!(SkillMaturity::L0Seed.score_weight(), 0);
        assert_eq!(SkillMaturity::L7Production.score_weight(), 100);
    }

    #[test]
    fn test_skill_result_constructors() {
        assert!(SkillResult::ok("done").is_ok());
        assert!(SkillResult::ok_with_id("created", 42).is_ok());
        assert!(SkillResult::error("fail").is_error());
        assert!(SkillResult::forbidden("nope").is_error());
        assert!(SkillResult::not_implemented("foo").is_error());
    }

    #[test]
    fn test_registry_loads_and_counts() {
        let registry = SkillRegistry::new();
        assert!(registry.count() > 0, "Registry should have skills registered");
    }

    #[test]
    fn test_registry_building_domain() {
        let registry = SkillRegistry::new();
        let building = registry.list_domain(SkillDomain::Building);
        assert!(building.len() >= 20, "Building domain should have 20+ skills, got {}", building.len());
    }

    #[test]
    fn test_registry_lookup_by_id() {
        let registry = SkillRegistry::new();
        let skill = registry.get("rez_box");
        assert!(skill.is_some(), "rez_box should be registered");
        let skill = skill.unwrap();
        assert_eq!(skill.domain, SkillDomain::Building);
        assert_eq!(skill.maturity, SkillMaturity::L7Production);
    }

    #[test]
    fn test_registry_find_by_domain_and_id() {
        let registry = SkillRegistry::new();
        assert!(registry.find(SkillDomain::Building, "rez_box").is_some());
        assert!(registry.find(SkillDomain::Scripting, "rez_box").is_none());
    }

    #[test]
    fn test_registry_search() {
        let registry = SkillRegistry::new();
        let results = registry.search("terrain");
        assert!(results.len() >= 4, "Search 'terrain' should find landscaping skills");
    }

    #[test]
    fn test_registry_dashboard() {
        let registry = SkillRegistry::new();
        let dash = registry.dashboard();
        assert_eq!(dash.total_domains, 14);
        assert!(dash.total_skills > 0);
    }

    #[test]
    fn test_generate_prompt_catalog() {
        let registry = SkillRegistry::new();
        let catalog = registry.generate_prompt_catalog();
        assert!(catalog.contains("SKILL ENGINE CATALOG"));
        assert!(catalog.contains("Building"));
        assert!(catalog.contains("Navigation"));
        assert!(catalog.contains("rez_box"));
        assert!(catalog.contains("teleport_agent"));
        assert!(catalog.contains("os_invoke_skill"));
        assert!(catalog.len() > 1000, "Catalog should be substantial, got {} bytes", catalog.len());
    }

    #[test]
    fn test_skill_def_to_info() {
        let registry = SkillRegistry::new();
        let skill = registry.get("rez_box").unwrap();
        let info = skill.to_info();
        assert_eq!(info.id, "rez_box");
        assert_eq!(info.domain, "building");
        assert!(!info.params.is_empty());
    }
}
