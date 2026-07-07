use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::openai::{ErrorBody, ErrorDetail};

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    #[allow(dead_code)]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, code) = match self {
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "jeton bearer manquant ou invalide".to_string(),
                Some("unauthorized".to_string()),
            ),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m, None),
        };
        let body = ErrorBody {
            error: ErrorDetail {
                message,
                kind: "invalid_request_error".into(),
                code,
            },
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn unauthorized_a_le_format_openai() {
        let resp = AppError::Unauthorized.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"]["type"], "invalid_request_error");
    }
}
