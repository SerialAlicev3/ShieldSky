use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;

pub const API_VERSION: &str = "sss-skyshield-api-v1";

#[derive(Debug, Clone, Serialize)]
pub struct ApiEnvelope<T> {
    pub api_version: &'static str,
    pub request_id: String,
    pub data: T,
}

impl<T> ApiEnvelope<T> {
    pub fn new(request_id: String, data: T) -> Self {
        Self {
            api_version: API_VERSION,
            request_id,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaceholderResponse {
    pub status: &'static str,
    pub endpoint: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    pub api_version: &'static str,
    pub request_id: String,
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn invalid_request(request_id: String, message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            request_id,
            "invalid_request",
            message,
        )
    }

    pub fn not_found(request_id: String, code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, request_id, code, message)
    }

    pub fn storage(request_id: String, message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "storage_error",
            message,
        )
    }

    pub fn insufficient_evidence(request_id: String, message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            request_id,
            "insufficient_evidence",
            message,
        )
    }

    pub fn response_not_authorized(request_id: String, message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            request_id,
            "response_not_authorized",
            message,
        )
    }

    fn new(
        status: StatusCode,
        request_id: String,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status,
            body: ApiErrorBody {
                api_version: API_VERSION,
                request_id,
                error: ApiErrorDetail {
                    code: code.to_string(),
                    message: message.into(),
                },
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
