//! Static file serving with SPA fallback
//!
//! This module provides handlers for serving SPA (Single Page Application) UI
//! static files from the Rust backend. Enable with SERVE_STATIC_FILES=true
//! and configure the directory with STATIC_FILES_DIR.

use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::path::PathBuf;

/// Creates a fallback handler that serves index.html for SPA routing.
///
/// When the Angular app uses client-side routing (e.g., `/admin/user-management`),
/// those routes don't exist as actual files on the server. This fallback ensures
/// that any route not matched by the API or static files serves index.html,
/// allowing Angular Router to handle the navigation.
pub fn spa_fallback_handler(static_dir: String) -> Router {
    let index_path = PathBuf::from(&static_dir).join("index.html");

    Router::new().fallback(get(move || {
        let index_path = index_path.clone();
        async move { serve_index_html(&index_path).await }
    }))
}

/// Serves the index.html file for SPA fallback
async fn serve_index_html(index_path: &PathBuf) -> Response {
    match tokio::fs::read(index_path).await {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(contents))
            .unwrap()
            .into_response(),
        Err(e) => {
            tracing::warn!("Failed to read index.html: {}", e);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(Body::from(
                    "Application not found. Please build the client first:\n\n\
                     cd client && npm run build\n\n\
                     Then copy the build output to the static directory.",
                ))
                .unwrap()
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_serve_index_html_success() {
        // Create a temporary directory with an index.html file
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.html");
        fs::write(&index_path, "<html><body>Test</body></html>").unwrap();

        let response = serve_index_html(&index_path).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "<html><body>Test</body></html>");
    }

    #[tokio::test]
    async fn test_serve_index_html_not_found() {
        let index_path = PathBuf::from("/nonexistent/path/index.html");

        let response = serve_index_html(&index_path).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
