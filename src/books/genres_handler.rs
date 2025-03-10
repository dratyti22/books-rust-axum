use crate::AppState;
use crate::books::model::Genres;
use crate::books::schema::GenresSchema;
use crate::service::response_server::{APIResult, ErrorResponse, SuccessResponse};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/v1/book/genres/create/",
    request_body = GenresSchema,
    responses(
        (status = 201, description = "Список жанров", body = Genres),
        (status = 400, description = "Ошибка валидации данных", body = ErrorResponse),
        (status = 500, description = "Ошибка сервера", body = ErrorResponse)
    ),
    security(
        ("Bearer" = ["admin"])
    ),
    tag = "Books genres"
)]
pub async fn create_genres(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GenresSchema>,
) -> APIResult<Genres> {
    if body.validate().is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid input data".to_string(),
                message: "Invalid input data".to_string(),
            }),
        ));
    }

    let genres = sqlx::query_as!(
        Genres,
        r#"INSERT INTO genres (name, description) VALUES ($1, $2) RETURNING *"#,
        body.name,
        body.description
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("Database error: {}", e),
            message: "Failed to create genre".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let response = SuccessResponse {
        data: genres,
        message: "Genre created successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/book/genres",
    responses(
        (status = 200, description = "Список жанров", body = Vec<Genres>),
        (status = 500, description = "Ошибка сервера", body = ErrorResponse)
    ),
    tag = "Books genres"
)]
pub async fn get_all_genres(State(data): State<Arc<AppState>>) -> APIResult<Vec<Genres>> {
    let genres = sqlx::query_as!(Genres, "SELECT * FROM genres")
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Database error: {}", e),
                message: "Failed to fetch all genres".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let response = SuccessResponse {
        data: genres,
        message: "Genres fetched successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}
