use crate::middleware::jwt_auth::JWTAuthMiddleware;
use crate::service::response_server::{APIResult, ErrorResponse, SuccessResponse};
use crate::users::model::{User, UserRole};
use crate::users::response::UserResponse;
use crate::users::schema::{LoginUserSchema, RegisterUserSchema};
use crate::users::token::{generate_jwt_token, verify_jwt_token, TokenDetails};
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::http::{header, HeaderMap, Response, StatusCode};
use axum::{Extension, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;
use std::sync::Arc;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/v1/user/register/",
    request_body = RegisterUserSchema,
    responses(
        (status = 201, description = "Успешно создано", body = UserResponse),
        (status = 400, description = "Ошибка валидации данных", body = ErrorResponse),
        (status = 409, description = "Ошибка такие данные уже есть", body = ErrorResponse),
        (status = 500, description = "Ошибка сервера", body = ErrorResponse)
    ),
    tag = "Users"
)]
pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> APIResult<UserResponse> {
    if body.validate().is_err() {
        let error = ErrorResponse {
            error: "Invalid input data".to_string(),
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
                    error: format!("Database error: {}", e),
                    message: "Request failed".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    if let Some(true) = user_exists {
        let error_response = ErrorResponse {
            error: "".to_string(),
            message: "Email already registered".to_string(),
        };
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Error while hashing password: {}", e),
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
        RETURNING id, first_name, last_name, middle_name, age, email, password, biography, file, verified, role as "role: UserRole", balance, rating, created_at, updated_at
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
            let error_response = ErrorResponse {
                error: format!("Database error: {}", e),
                message: "Error when adding to the database record".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let response = SuccessResponse {
        data: UserResponse::new(&user),
        message: "Registration successful".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/api/v1/user/login/",
    request_body = LoginUserSchema,
    responses(
        (status = 200, description = "Успешно авторизирован", body = UserResponse),
        (status = 400, description = "Ошибка валидации данных или ошибка хеширования пароля", body = ErrorResponse),
        (status = 409, description = "Ошибка такие данные уже есть", body = ErrorResponse),
        (status = 500, description = "Ошибка сервера", body = ErrorResponse)
    ),
    tag = "Users"
)]
pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> APIResult<String> {
    if body.validate().is_err() {
        let error = ErrorResponse {
            error: "".to_string(),
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
        biography,
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
            error: format!("Database error: {}", e),
            message: "Request failed".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?
    .ok_or_else(|| {
        let error_response = ErrorResponse {
            error: "".to_string(),
            message: "Password hashing failed".to_string(),
        };
        (StatusCode::BAD_REQUEST, Json(error_response))
    })?;

    let valid_password = match PasswordHash::new(&user.password) {
        Ok(hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &hash).is_ok(),
        Err(_) => false,
    };

    if !valid_password {
        let error_response = ErrorResponse {
            error: "".to_string(),
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

    let json_response = SuccessResponse {
        data: "success".to_string(),
        message: "Login successful".to_string(),
    };

    let mut response = Response::new(json_response.to_string());
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
    Ok((StatusCode::OK, Json(json_response)))
}

#[utoipa::path(
    post,
    path = "/api/v1/user/logout/",
    responses(
        (status = 200, description = "Успешно вышли", body = String),
        (status = 403, description = "Ошибка авторизации", body = ErrorResponse),
        (status = 401, description = "Ошибка проверка токена", body = ErrorResponse),
        (status = 500, description = "Ошибка сервера", body = ErrorResponse)
    ),
    security(
        ("Bearer" = ["user"])
    ),
    tag = "Users"
)]
pub async fn logout_user_handler(
    cookie_jar: CookieJar,
    Extension(auth_guard): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> APIResult<String> {
    let message = "Token is invalid or session has expired".to_string();

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            let error_response = ErrorResponse {
                error: "".to_string(),
                message,
            };
            (StatusCode::FORBIDDEN, Json(error_response))
        })?;

    let refresh_token_details =
        match verify_jwt_token(data.env.refresh_token_public_key.to_owned(), &refresh_token) {
            Ok(token_details) => token_details,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("{:?}", e),
                    message: "Error verifying token".to_string(),
                };
                return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
            }
        };

    let mut redis_client = data
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Redis error: {}", e),
                message: "Redis error".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    redis_client
        .del::<_, ()>(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.accesses_token_uuid.to_string(),
        ])
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Redis Error 2: {:?}", e),
                message: "Redis Error".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true);
    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false);

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

    let response_success = SuccessResponse {
        data: "success".to_string(),
        message: "Logout successful".to_string(),
    };

    let mut response = Response::new(response_success.to_string());
    response.headers_mut().extend(headers);
    Ok((StatusCode::OK, Json(response_success)))
}

fn generate_token(
    user_id: uuid::Uuid,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, (StatusCode, Json<ErrorResponse>)> {
    generate_jwt_token(user_id, max_age, private_key).map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("error generating token: {}", e),
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
                error: format!("Redis error Save Token: {}", e),
                message: "Redis error".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    redis_client
        .set_ex::<_, _, ()>(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            max_age as u64,
        )
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Redis error Save Token 2: {}", e),
                message: "Redis error".to_string(),
            };
            (StatusCode::UNPROCESSABLE_ENTITY, Json(error_response))
        })?;
    Ok(())
}
