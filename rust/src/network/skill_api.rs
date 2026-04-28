use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;

use crate::ai::skill_engine::{RegistryDashboard, SkillDomain, SkillInfo, SkillRegistry};

fn registry() -> SkillRegistry {
    SkillRegistry::new()
}

fn resolve_domain(name: &str) -> Option<SkillDomain> {
    SkillDomain::all().iter().find(|d| d.id() == name).copied()
}

#[derive(Serialize)]
struct DomainSummary {
    id: String,
    display_name: String,
    skill_count: usize,
    score: u32,
}

#[derive(Serialize)]
struct SkillListResponse {
    total_skills: usize,
    total_domains: usize,
    domains: Vec<DomainWithSkills>,
}

#[derive(Serialize)]
struct DomainWithSkills {
    id: String,
    display_name: String,
    skills: Vec<SkillInfo>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn list_all_skills() -> Json<SkillListResponse> {
    let reg = registry();
    let domains: Vec<DomainWithSkills> = SkillDomain::all()
        .iter()
        .map(|d| {
            let skills = reg.list_domain(*d).iter().map(|s| s.to_info()).collect();
            DomainWithSkills {
                id: d.id().to_string(),
                display_name: d.display_name().to_string(),
                skills,
            }
        })
        .collect();
    let total_skills: usize = domains.iter().map(|d| d.skills.len()).sum();
    Json(SkillListResponse {
        total_skills,
        total_domains: domains.len(),
        domains,
    })
}

async fn get_dashboard() -> Json<RegistryDashboard> {
    let reg = registry();
    Json(reg.dashboard())
}

async fn list_domain_skills(Path(domain): Path<String>) -> impl IntoResponse {
    let Some(d) = resolve_domain(&domain) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Unknown domain: {}", domain)})),
        )
            .into_response();
    };
    let reg = registry();
    let skills: Vec<SkillInfo> = reg.list_domain(d).iter().map(|s| s.to_info()).collect();
    let dash = reg.domain_dashboard(d);
    Json(serde_json::json!({
        "domain": d.id(),
        "display_name": d.display_name(),
        "total_skills": skills.len(),
        "score": dash.score,
        "by_level": dash.by_level,
        "skills": skills,
    }))
    .into_response()
}

async fn get_skill(Path((domain, skill_id)): Path<(String, String)>) -> impl IntoResponse {
    let Some(d) = resolve_domain(&domain) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Unknown domain: {}", domain)})),
        )
            .into_response();
    };
    let reg = registry();
    let Some(skill) = reg.find(d, &skill_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Skill '{}' not found in domain '{}'", skill_id, domain)})),
        ).into_response();
    };
    Json(skill.to_info()).into_response()
}

async fn search_skills(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();
    if query.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing query parameter 'q'"})),
        )
            .into_response();
    }
    let reg = registry();
    let results: Vec<SkillInfo> = reg.search(&query).iter().map(|s| s.to_info()).collect();
    Json(serde_json::json!({
        "query": query,
        "count": results.len(),
        "results": results,
    }))
    .into_response()
}

pub fn create_skill_api_router() -> Router {
    Router::new()
        .route("/skills", get(list_all_skills))
        .route("/skills/dashboard", get(get_dashboard))
        .route("/skills/search", get(search_skills))
        .route("/skills/{domain}", get(list_domain_skills))
        .route("/skills/{domain}/{skill_id}", get(get_skill))
}
