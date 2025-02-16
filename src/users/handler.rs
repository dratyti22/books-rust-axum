use crate::users::model::{User, UserRole};
use crate::users::response::{ErrorResponse, UserResponse};
use crate::users::schema::{LoginUserSchema, RegisterUserSchema};
use crate::users::token::{generate_jwt_token, TokenDetails};
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::http::{header, HeaderMap, Response, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, SameSite};
use redis::AsyncCommands;
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

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if body.validate().is_err() {
        let error = ErrorResponse {
            data: None,
            message: "Invalid input data".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error)));
    }

    let user: User = sqlx::query_as!(
        User,
        r#"
        SELECT
        id,
        first_name,
        last_name,
        middle_name,
        age,
        email,
        password,
        file,
        verified,
        role as "role: UserRole",
        balance,
        rating,
        created_at,
        updated_at
        FROM users
        WHERE email = $1
        "#,
        body.email
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        let error_response = ErrorResponse {
            data: Some(format!("Database error: {}", e)),
            message: "Request failed".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?
    .ok_or_else(|| {
        let error_response = ErrorResponse {
            data: None,
            message: "Password hashing failed".to_string(),
        };
        (StatusCode::BAD_REQUEST, Json(error_response))
    })?;

    let valid_password = match PasswordHash::new(&user.password) {
        Ok(hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !valid_password {
        let error_response = ErrorResponse {
            data: None,
            message: "Invalid email or password".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let access_token_details = generate_token(
        user.id,
        data.env.access_token_max_age,
        data.env.access_token_private_key.to_owned(),
    )?;

    let refresh_token_details = generate_token(
        user.id,
        data.env.refresh_token_max_age,
        data.env.refresh_token_private_key.to_owned(),
    )?;
    save_token_data_to_redis(&data, &access_token_details, data.env.access_token_max_age).await?;
    save_token_data_to_redis(
        &data,
        &refresh_token_details,
        data.env.refresh_token_max_age,
    )
    .await?;

    let access_cookie = Cookie::build((
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::days(data.env.access_token_max_age))
    .same_site(SameSite::Lax)
    .http_only(true);

    let refresh_cookie = Cookie::build((
        "refresh_token",
        refresh_token_details.token.unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.refresh_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::days(data.env.access_token_max_age))
        .same_site(SameSite::Lax)
        .http_only(false);

    let mut response = Response::new(
        json!({"status": "success", "message": "Login successful"})
            .to_string(),
    );
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    response.headers_mut().extend(headers);
    Ok(response)
}

fn generate_token(
    user_id: uuid::Uuid,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, (StatusCode, Json<ErrorResponse>)> {
    generate_jwt_token(user_id, max_age, private_key).map_err(|e| {
        let error_response = ErrorResponse {
            data: Some(format!("error generating token: {}", e)),
            message: "Error token".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
}

async fn save_token_data_to_redis(
    data: &Arc<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let mut redis_client = data
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                data: None,
                message: format!("Redis error: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    redis_client
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            max_age as u64,
        )
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                data: None,
                message: format!("Redis error: {}", e),
            };
            (StatusCode::UNPROCESSABLE_ENTITY, Json(error_response))
        })?;
    Ok(())
}
