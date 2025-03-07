use crate::books::model::Books;
use rust_decimal::Decimal;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct BookResponse {
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
    pub discounted_price: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BookResponse {
    pub fn from_book(book: Books) -> Self {
        let discounted_price = book.price * (Decimal::ONE - book.discount / Decimal::from(100));

        Self {
            id: book.id,
            title: book.title,
            description: book.description,
            author_id: book.author_id,
            genre_id: book.genre_id,
            publication_year: book.publication_year,
            isbn: book.isbn,
            cover_image: book.cover_image,
            price: book.price,
            discount: book.discount,
            discounted_price,
            created_at: book.created_at,
            updated_at: book.updated_at,
        }
    }
}
