use axum::Json;
use axum::http::StatusCode;
use serde::Serialize;
use std::fmt::Formatter;
use utoipa::ToSchema;

pub type APIResult<T> =
    Result<(StatusCode, Json<SuccessResponse<T>>), (StatusCode, Json<ErrorResponse>)>;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse<T: Serialize> {
    #[schema(example = "Some data")]
    pub data: T,
    #[schema(example = "Operation successful")]
    pub message: String,
}

impl<T: Serialize> std::fmt::Display for SuccessResponse<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json)
    }
}
