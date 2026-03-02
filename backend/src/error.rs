use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: ErrorBody {
                code: status.as_u16(),
                message: self.to_string(),
            },
        };
        (status, axum::Json(body)).into_response()
    }
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn error_to_status_and_body(error: AppError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn bad_request_returns_400() {
        let (status, body) = error_to_status_and_body(AppError::BadRequest("invalid email".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], 400);
        assert_eq!(body["error"]["message"], "Bad request: invalid email");
    }

    #[tokio::test]
    async fn unauthorized_returns_401() {
        let (status, body) = error_to_status_and_body(AppError::Unauthorized("invalid token".into())).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], 401);
        assert_eq!(body["error"]["message"], "Unauthorized: invalid token");
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let (status, body) = error_to_status_and_body(AppError::NotFound("project not found".into())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], 404);
        assert_eq!(body["error"]["message"], "Not found: project not found");
    }

    #[tokio::test]
    async fn conflict_returns_409() {
        let (status, body) = error_to_status_and_body(AppError::Conflict("email already exists".into())).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], 409);
        assert_eq!(body["error"]["message"], "Conflict: email already exists");
    }

    #[tokio::test]
    async fn internal_error_returns_500() {
        let (status, body) = error_to_status_and_body(AppError::Internal("database error".into())).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], 500);
        assert_eq!(body["error"]["message"], "Internal server error: database error");
    }
}
