use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct GenresSchema {
    #[validate(length(min = 1))]
    pub name: String,

    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct BookSchema {
    pub title: String,
    pub description: String,
    pub genre_id: uuid::Uuid,
    pub isbn: String,
    pub cover_image: Option<String>,
    pub price: i64,
    pub discount: i64,
}
