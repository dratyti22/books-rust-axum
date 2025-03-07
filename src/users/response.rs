use crate::users::model::{User, UserRole};
use rust_decimal::Decimal;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub age: i32,
    pub email: String,
    pub biography: Option<String>,
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
            biography: user.biography.clone(),
            verified: user.verified,
            role: user.role.to_owned(),
            balance: user.balance,
            rating: user.rating,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
