use crate::users::model::{User, UserRole};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub data: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub age: i32,
    pub email: String,
    pub file: String,
    pub verified: bool,
    pub role: UserRole,
    pub balance: Decimal,
    pub rating: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserResponse {
    pub fn new(user: &User) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name.to_owned(),
            last_name: user.last_name.to_owned(),
            middle_name: user.middle_name.to_owned(),
            age: user.age,
            email: user.email.to_owned(),
            file: user.file.clone(),
            verified: user.verified,
            role: user.role.to_owned(),
            balance: user.balance,
            rating: user.rating,
            created_at: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: user.created_at.unwrap_or_else(|| chrono::Utc::now()),
        }
    }
}
