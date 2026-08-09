use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{PermissionManager, ToolRuleSet, KNOWN_TOOLS};
use super::rules::ToolPermission;

#[derive(Deserialize)]
pub struct SetRuleRequest {
    pub tool: String,
    pub permission: ToolPermission,
    #[serde(default)]
    pub note: String,
}

#[derive(Deserialize)]
pub struct SetDefaultRequest {
    pub permission: ToolPermission,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    #[serde(default = "dl")]
    pub limit: usize,
}

fn dl() -> usize {
    100
}

#[derive(Serialize)]
pub struct PermissionOverview {
    pub rules: ToolRuleSet,
    pub known_tools: Vec<ToolInfo>,
}

#[derive(Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

pub async fn get_permissions(State(pm): State<PermissionManager>) -> Json<PermissionOverview> {
    let r = pm.get_rules().await;
    Json(PermissionOverview {
        known_tools: KNOWN_TOOLS
            .iter()
            .map(|(n, d)| ToolInfo {
                name: n.to_string(),
                description: d.to_string(),
            })
            .collect(),
        rules: r,
    })
}

pub async fn update_permissions(
    State(pm): State<PermissionManager>,
    Json(r): Json<ToolRuleSet>,
) -> Result<StatusCode, (StatusCode, String)> {
    for e in &r.tools {
        pm.set_tool_permission(&e.tool, e.permission.clone(), e.note.clone())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    pm.set_default_mode(r.default_mode)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn get_rules(State(pm): State<PermissionManager>) -> Json<ToolRuleSet> {
    Json(pm.get_rules().await)
}

pub async fn set_rule(
    State(pm): State<PermissionManager>,
    Json(r): Json<SetRuleRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    pm.set_tool_permission(&r.tool, r.permission, r.note)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn remove_rule(
    State(pm): State<PermissionManager>,
    axum::extract::Path(tool): axum::extract::Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    pm.remove_tool_rule(&tool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn set_default_permission(
    State(pm): State<PermissionManager>,
    Json(r): Json<SetDefaultRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    pm.set_default_mode(r.permission)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn reset_permissions(
    State(pm): State<PermissionManager>,
) -> Result<StatusCode, (StatusCode, String)> {
    pm.reset_to_defaults()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn get_audit_log(
    State(pm): State<PermissionManager>,
    axum::extract::Query(q): axum::extract::Query<AuditQuery>,
) -> Json<Vec<super::AuditEntry>> {
    Json(pm.get_audit_log(q.limit).await)
}

pub async fn get_audit_stats(
    State(pm): State<PermissionManager>,
) -> Json<super::audit::AuditStats> {
    let e = pm.get_audit_log(1000).await;
    let mut s = super::audit::AuditStats::default();
    for x in &e {
        s.total += 1;
        if x.decision.contains("Allow") {
            s.allowed += 1;
        } else if x.decision.contains("Deny") {
            s.denied += 1;
        } else if x.decision.contains("Ask") {
            s.asked += 1;
        }
        *s.by_tool.entry(x.tool.clone()).or_insert(0) += 1;
    }
    Json(s)
}

pub async fn clear_audit_log(
    State(pm): State<PermissionManager>,
) -> Result<StatusCode, (StatusCode, String)> {
    pm.clear_audit_log()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn list_known_tools() -> Json<Vec<ToolInfo>> {
    Json(
        KNOWN_TOOLS
            .iter()
            .map(|(n, d)| ToolInfo {
                name: n.to_string(),
                description: d.to_string(),
            })
            .collect(),
    )
}