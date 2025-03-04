use rust_decimal::Decimal;
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
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    #[validate(length(equal = 13))]
    pub isbn: String,
    pub discount: Option<Decimal>,
    pub genre_id: uuid::Uuid,
    pub cover_image: String,
}


#[derive(Debug, Validate, Deserialize)]
pub struct BookUpdateSchema {
    #[validate(length(min=1))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub price: Option<Decimal>,
    pub discount: Option<Decimal>,
}
