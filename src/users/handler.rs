use crate::users::model::{User, UserRole};
use crate::users::response::{ErrorResponse, UserResponse};
use crate::users::schema::RegisterUserSchema;
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if body.validate().is_err() {
        let error = ErrorResponse {
            data: None,
            message: "Invalid input data".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error)));
    }

    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(body.email.to_owned())
            .fetch_one(&data.db)
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    data: Some(format!("Database error: {}", e)),
                    message: "Request failed".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    if let Some(true) = user_exists {
        let error_response = ErrorResponse {
            data: None,
            message: "Email already registered".to_string(),
        };
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = ErrorResponse {
                data: Some(format!("Error while hashing password: {}", e)),
                message: "Password hashing failed".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (first_name, last_name, middle_name, age, email, password)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, first_name, last_name, middle_name, age, email, password, file, verified, role as "role: UserRole", balance, rating, created_at, updated_at
        "#,
        body.first_name,
        body.last_name,
        body.middle_name,
        body.age,
        body.email,
        hashed_password
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        let error_response = ErrorResponse{
            data: None,
            message: format!("Database error: {}", e)
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let response = json!({
        "data": json!({"user": UserResponse::new(&user)}),
        "message": "Registration successful".to_string(),
    });

    Ok((StatusCode::CREATED, Json(response)))
}
