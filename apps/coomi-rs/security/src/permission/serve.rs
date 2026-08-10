use std::net::SocketAddr;
use std::path::PathBuf;

use super::manager::PermissionManager;
use super::router::permission_routes;

/// 启动权限管理 Web 服务
///
/// # 参数
/// - `addr`: 监听地址，如 `0.0.0.0:8900`
/// - `config_dir`: 配置目录，用于存储 rules.json 和 audit.log
pub async fn serve(addr: SocketAddr, config_dir: PathBuf) -> anyhow::Result<()> {
    // 确保配置目录存在
    tokio::fs::create_dir_all(&config_dir).await?;

    let config_path = config_dir.join("rules.json");
    let audit_path = config_dir.join("audit.log");

    let pm = PermissionManager::new(config_path, audit_path);
    let app = permission_routes(pm);

    println!("🔒 Agent Permission Manager");
    println!("   http://{}", addr);
    println!("   配置目录: {}", config_dir.display());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 启动服务（带默认配置）
///
/// 默认监听 `0.0.0.0:8900`，配置目录 `~/.coomi/permissions`
pub async fn serve_default() -> anyhow::Result<()> {
    let addr: SocketAddr = "0.0.0.0:8900".parse()?;
    let config_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".coomi")
        .join("permissions");
    serve(addr, config_dir).await
}