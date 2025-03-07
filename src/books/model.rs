use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Genres {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Books {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub author_id: uuid::Uuid,
    pub genre_id: uuid::Uuid,
    pub publication_year: Option<i16>,
    pub isbn: String,
    pub cover_image: Option<String>,
    pub price: Decimal,
    pub discount: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
