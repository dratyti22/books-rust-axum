use crate::users::model::{User, UserRole};
use crate::users::response::ErrorResponse;
use crate::users::token::verify_jwt_token;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTAuthMiddleware {
    pub user: User,
    pub accesses_token_uuid: uuid::Uuid,
}
pub async fn examination_auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
) -> Result<Request<Body>, (StatusCode, Json<ErrorResponse>)> {
    let access_token = cookie_jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let access_token = access_token.ok_or_else(|| {
        let json_error = ErrorResponse {
            data: None,
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let access_token_details =
        match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
            Ok(token_details) => token_details,
            Err(e) => {
                let error_response = ErrorResponse {
                    data: None,
                    message: format!("{:?}", e),
                };
                return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
            }
        };

    let access_token_uuid = uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string())
        .map_err(|_| {
            let error_response = ErrorResponse {
                data: None,
                message: "Invalid token".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;

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

    let redis_token_user_id = redis_client
        .get::<_, String>(access_token_uuid.clone().to_string())
        .await
        .map_err(|_| {
            let error_response = ErrorResponse {
                data: None,
                message: "Token is invalid or session has expired".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;

    let user_id = uuid::Uuid::parse_str(&redis_token_user_id).map_err(|_| {
        let json_error = ErrorResponse {
            data: None,
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;
    let user = sqlx::query_as!(
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
        FROM users WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        let json_error = ErrorResponse {
            data: None,
            message: format!("Error fetching user from database: {}", e),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
    })?;

    let user = user.ok_or_else(|| {
        let json_error = ErrorResponse {
            data: None,
            message: "The user belonging to this token no longer exists".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;
    req.extensions_mut().insert(JWTAuthMiddleware {
        user,
        accesses_token_uuid: access_token_uuid,
    });
    Ok(req)
}

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let auth_result = examination_auth(cookie_jar, State(data), req).await;
    match auth_result {
        Ok(response) => Ok(next.run(response).await),
        Err(err) => Err(err),
    }
}

pub async fn auth_roles(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
    allowed_roles: Vec<UserRole>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let auth_result = examination_auth(cookie_jar, State(data), req).await;

    match auth_result {
        Ok(req) => {
            if let Some(auth_middleware) = req.extensions().get::<JWTAuthMiddleware>().cloned() {
                if allowed_roles.contains(&auth_middleware.user.role) {
                    Ok(next.run(req).await)
                } else {
                    let json_error = ErrorResponse {
                        data: None,
                        message: "You do not have permission to access this resource".to_string(),
                    };
                    Err((StatusCode::FORBIDDEN, Json(json_error)))
                }
            } else {
                let json_error = ErrorResponse {
                    data: None,
                    message: "Authentication failed".to_string(),
                };
                Err((StatusCode::UNAUTHORIZED, Json(json_error)))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn auth_admin(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    auth_roles(cookie_jar, State(data), req, next, vec![UserRole::Admin]).await
}

pub async fn auth_author_worker_admin(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    auth_roles(
        cookie_jar,
        State(data),
        req,
        next,
        vec![UserRole::Author, UserRole::Worker, UserRole::Admin],
    )
    .await
}
