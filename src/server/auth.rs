//! Middleware d'authentification bearer.

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::error::AppError;
use super::AppState;

pub async fn require_bearer(State(state): State<AppState>, request: Request, next: Next) -> Response {
    match &state.config.server.bearer_token {
        Some(tok) if !tok.is_empty() => {
            let expected = format!("Bearer {tok}");
            let ok = request
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .map(|h| h == expected)
                .unwrap_or(false);
            if !ok {
                return AppError::Unauthorized.into_response();
            }
        }
        _ => {} // bearer vide/None -> auth désactivée
    }
    next.run(request).await
}
