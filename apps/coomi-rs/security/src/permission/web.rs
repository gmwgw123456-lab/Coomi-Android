use axum::http::{header, StatusCode};
use axum::response::Html;

/// 管理界面主页
pub async fn index_page() -> Html<&'static str> {
    Html(include_str!("web/index.html"))
}

/// CSS 样式
pub async fn css() -> (StatusCode, [(header::HeaderName, &'static str); 1], &'static str) {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("web/style.css"),
    )
}

/// JavaScript
pub async fn js() -> (StatusCode, [(header::HeaderName, &'static str); 1], &'static str) {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("web/app.js"),
    )
}