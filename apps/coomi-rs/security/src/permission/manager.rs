use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::audit::{AuditEntry, AuditLogger};
use super::rules::{ToolPermission, ToolRule, ToolRuleSet};

/// Known tools with their descriptions
pub const KNOWN_TOOLS: &[(&str, &str)] = &[
    ("filesystem-read", "读取文件内容"),
    ("filesystem-create", "创建新文件"),
    ("filesystem-edit", "编辑文件内容"),
    ("filesystem-replaceedit", "搜索替换编辑文件"),
    ("terminal-execute", "执行终端命令"),
    ("ace-search", "代码搜索"),
    ("websearch-search", "网络搜索"),
    ("websearch-fetch", "获取网页内容"),
];

pub struct PermissionManager {
    rules: Arc<RwLock<ToolRuleSet>>,
    audit: Arc<AuditLogger>,
    config_path: PathBuf,
}

impl Clone for PermissionManager {
    fn clone(&self) -> Self {
        Self {
            rules: Arc::clone(&self.rules),
            audit: Arc::clone(&self.audit),
            config_path: self.config_path.clone(),
        }
    }
}

impl PermissionManager {
    pub fn new(config_path: PathBuf, audit_path: PathBuf) -> Self {
        let rules = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default();

        Self {
            rules: Arc::new(RwLock::new(rules)),
            audit: Arc::new(AuditLogger::new(&audit_path)),
            config_path,
        }
    }

    pub async fn get_rules(&self) -> ToolRuleSet {
        self.rules.read().await.clone()
    }

    pub async fn set_tool_permission(
        &self,
        tool: &str,
        permission: ToolPermission,
        note: String,
    ) -> Result<()> {
        let mut rules = self.rules.write().await;
        if let Some(rule) = rules.tools.iter_mut().find(|r| r.tool == tool) {
            rule.permission = permission;
            rule.note = note;
        } else {
            rules.tools.push(ToolRule {
                tool: tool.to_string(),
                permission,
                note,
            });
        }
        self.save_rules(&rules).await?;
        Ok(())
    }

    pub async fn remove_tool_rule(&self, tool: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.tools.retain(|r| r.tool != tool);
        self.save_rules(&rules).await?;
        Ok(())
    }

    pub async fn set_default_mode(&self, permission: ToolPermission) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.default_mode = permission;
        self.save_rules(&rules).await?;
        Ok(())
    }

    pub async fn reset_to_defaults(&self) -> Result<()> {
        let mut rules = self.rules.write().await;
        *rules = ToolRuleSet::default();
        self.save_rules(&rules).await?;
        Ok(())
    }

    pub async fn check_permission(&self, tool: &str) -> ToolPermission {
        let rules = self.rules.read().await;
        rules
            .tools
            .iter()
            .find(|r| r.tool == tool)
            .map(|r| r.permission.clone())
            .unwrap_or_else(|| rules.default_mode.clone())
    }

    pub async fn log_audit(&self, entry: &AuditEntry) {
        self.audit.log(entry).await;
    }

    pub async fn get_audit_log(&self, limit: usize) -> Vec<AuditEntry> {
        self.audit.read_recent(limit).await
    }

    pub async fn clear_audit_log(&self) -> Result<()> {
        self.audit.clear().await?;
        Ok(())
    }

    async fn save_rules(&self, rules: &ToolRuleSet) -> Result<()> {
        let json = serde_json::to_string_pretty(rules)?;
        tokio::fs::write(&self.config_path, json).await?;
        Ok(())
    }
}