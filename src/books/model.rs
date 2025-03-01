use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Genres {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Books {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub author_id: uuid::Uuid,
    pub genre_id: uuid::Uuid,
    pub publication_year: i32,
    pub isbn: String,
    pub cover_image: String,
    pub price: i64,
    pub discount: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
