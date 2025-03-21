use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterUserSchema {
    #[validate(length(min = 2, max = 100))]
    pub first_name: String,

    #[validate(length(min = 2, max = 100))]
    pub last_name: String,

    #[validate(length(min = 2, max = 100))]
    pub middle_name: Option<String>,

    #[validate(range(min = 0, max = 150))]
    pub age: i32,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginUserSchema {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}
