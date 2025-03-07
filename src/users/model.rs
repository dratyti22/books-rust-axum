use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::Display;

#[derive(
    Debug, Clone, Display, Serialize, Deserialize, sqlx::Type, PartialEq, utoipa::ToSchema,
)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    #[sqlx(rename = "пользователь")]
    User,
    #[sqlx(rename = "автор")]
    Author,
    #[sqlx(rename = "работник")]
    Worker,
    #[sqlx(rename = "админ")]
    Admin,
    #[sqlx(rename = "продавец")]
    Seller,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub age: i32,
    pub email: String,
    pub password: String,
    pub biography: Option<String>,
    pub file: String,
    pub verified: bool,
    pub role: UserRole,
    pub balance: Decimal,
    pub rating: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
