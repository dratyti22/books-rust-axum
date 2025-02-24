use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Genres {
    id: uuid::Uuid,
    name: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Books {
    id: uuid::Uuid,
    title: String,
    description: String,
    author_id: uuid::Uuid,
    genre_id: uuid::Uuid,
    publication_year: i32,
    isbn: String,
    cover_image: String,
    price: i64,
    discount: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
