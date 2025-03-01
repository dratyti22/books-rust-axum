use crate::books::model::Genres;
use crate::books::schema::GenresSchema;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

pub async fn create_genres(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GenresSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    if body.validate().is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid input data"
            })),
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
        let error_response = json!({
            "error": format!("Database error: {}", e),
            "message": "Failed to create genre"
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let response = json!({
        "data": genres,
        "message": "Genre created successfully"
    });

    Ok(Json(response))
}
