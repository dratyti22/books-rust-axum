use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct GenresSchema {
    #[validate(length(min = 1))]
    pub name: String,

    #[validate(length(min = 1))]
    pub description: String,
}
