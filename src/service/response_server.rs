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
