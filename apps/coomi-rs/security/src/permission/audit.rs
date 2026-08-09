use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub tool: String,
    pub decision: String,
    pub rule: String,
    pub context: String,
    pub arguments_summary: String,
}

pub struct AuditLogger {
    path: PathBuf,
    cache: RwLock<Vec<AuditEntry>>,
}

impl AuditLogger {
    pub fn new(path: &PathBuf) -> Self {
        let cache = std::fs::read_to_string(path)
            .ok()
            .map(|c| {
                c.lines()
                    .filter_map(|l| serde_json::from_str(l).ok())
                    .collect()
            })
            .unwrap_or_default();
        Self {
            path: path.clone(),
            cache: RwLock::new(cache),
        }
    }

    pub async fn log(&self, entry: &AuditEntry) {
        let line = serde_json::to_string(entry).unwrap_or_default();
        let _ = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map(|mut f| {
                use tokio::io::AsyncWriteExt;
                async move {
                    let _ = f.write_all(format!("{}\n", line).as_bytes()).await;
                }
            });
        let mut c = self.cache.write().await;
        c.push(entry.clone());
        if c.len() > 1000 {
            let excess = c.len() - 1000;
            c.drain(..excess);
        }
    }

    pub async fn read_recent(&self, limit: usize) -> Vec<AuditEntry> {
        let c = self.cache.read().await;
        c[c.len().saturating_sub(limit)..].to_vec()
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        let mut c = self.cache.write().await;
        c.clear();
        tokio::fs::write(&self.path, "").await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStats {
    pub total: usize,
    pub allowed: usize,
    pub denied: usize,
    pub asked: usize,
    pub by_tool: std::collections::HashMap<String, usize>,
}