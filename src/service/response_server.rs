use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub data: T,
    pub message: String,
}

pub type APIResult<T> =
    Result<(StatusCode, Json<SuccessResponse<T>>), (StatusCode, Json<ErrorResponse>)>;
