use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

pub const API_VERSION: &str = "sss-api-v1";

#[derive(Debug, Clone, Serialize)]
pub struct ApiEnvelope<T>
where
    T: Serialize,
{
    pub request_id: String,
    pub api_version: &'static str,
    pub data: T,
}

impl<T> ApiEnvelope<T>
where
    T: Serialize,
{
    #[must_use]
    pub fn new(request_id: String, data: T) -> Self {
        Self {
            request_id,
            api_version: API_VERSION,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorEnvelope {
    pub request_id: String,
    pub api_version: &'static str,
    pub error: ApiErrorBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    request_id: String,
    code: &'static str,
    message: String,
    retryable: bool,
}

impl ApiError {
    #[must_use]
    pub fn not_found(request_id: String, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            request_id,
            code,
            message: message.into(),
            retryable: false,
        }
    }

    #[must_use]
    pub fn storage(request_id: String, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            code: "storage_error",
            message: message.into(),
            retryable: true,
        }
    }

    #[must_use]
    pub fn invalid_request(request_id: String, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            request_id,
            code: "invalid_request",
            message: message.into(),
            retryable: false,
        }
    }

    #[must_use]
    pub fn source_unavailable(request_id: String, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            request_id,
            code: "source_unavailable",
            message: message.into(),
            retryable: true,
        }
    }

    #[must_use]
    pub fn nasa_api_unavailable(request_id: String, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            request_id,
            code: "nasa_api_unavailable",
            message: message.into(),
            retryable: true,
        }
    }

    #[must_use]
    pub fn notification_unavailable(request_id: String, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            request_id,
            code: "notification_unavailable",
            message: message.into(),
            retryable: true,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorEnvelope {
            request_id: self.request_id,
            api_version: API_VERSION,
            error: ApiErrorBody {
                code: self.code,
                message: self.message,
                retryable: self.retryable,
            },
        };
        (self.status, Json(body)).into_response()
    }
}
