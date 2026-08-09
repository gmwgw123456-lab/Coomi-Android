use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolPermission {
    Allow,
    Deny,
    Ask,
}

impl ToolPermission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "Allow",
            Self::Deny => "Deny",
            Self::Ask => "Ask",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRule {
    pub tool: String,
    pub permission: ToolPermission,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRuleSet {
    pub default_mode: ToolPermission,
    pub tools: Vec<ToolRule>,
}

impl Default for ToolRuleSet {
    fn default() -> Self {
        Self {
            default_mode: ToolPermission::Ask,
            tools: Vec::new(),
        }
    }
}