use axum::routing::{delete, get, post, put};
use axum::Router;

use super::api;
use super::manager::PermissionManager;

/// 构建权限管理 API 路由
pub fn permission_routes(pm: PermissionManager) -> Router {
    Router::new()
        // 权限管理
        .route("/permissions", get(api::get_permissions))
        .route("/permissions", put(api::update_permissions))
        .route("/permissions/reset", post(api::reset_permissions))
        // 规则管理
        .route("/rules", get(api::get_rules))
        .route("/rules", post(api::set_rule))
        .route("/rules/:tool", delete(api::remove_rule))
        // 默认权限
        .route("/default", post(api::set_default_permission))
        // 审计日志
        .route("/audit", get(api::get_audit_log))
        .route("/audit", delete(api::clear_audit_log))
        .route("/audit/stats", get(api::get_audit_stats))
        // 工具列表
        .route("/tools", get(api::list_known_tools))
        // Web 管理界面
        .route("/", get(super::web::index_page))
        .route("/static/style.css", get(super::web::css))
        .route("/static/app.js", get(super::web::js))
        .with_state(pm)
}