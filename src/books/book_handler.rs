use crate::books::schema::BookSchema;
use crate::middleware::jwt_auth::JWTAuthMiddleware;
use crate::service::response_server::{ErrorResponse, SuccessResponse};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum_macros::debug_handler;
use chrono::Datelike;
use std::sync::Arc;

#[debug_handler]
pub async fn create_book(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<BookSchema>,
) -> Result<(StatusCode, Json<SuccessResponse<String>>), (StatusCode, Json<ErrorResponse>)> {
    let user_id = user.user.id;
    let image = body
        .cover_image
        .unwrap_or_else(|| "uploads/books/default.jpg".to_string());
    let publication_year: i32 = chrono::Utc::now().year();
    let query_result = sqlx::query(
        r#"INSERT INTO books (title, description, author_id, genre_id, isbn, cover_image, price, discount, publication_year)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"#,
    )
        .bind(body.title.to_owned())
        .bind(body.description.to_owned())
        .bind(user_id.to_owned())
        .bind(body.genre_id.to_owned())
        .bind(body.isbn.to_owned())
        .bind(image.to_owned())
        .bind(body.price)
        .bind(body.discount)
        .bind(publication_year)
        .execute(&data.db)
        .await
        .map_err(|e: sqlx::Error| e.to_string());

    if let Err(e) = query_result {
        if e.contains("Duplicate entry") {
            let e = ErrorResponse {
                error: "".to_string(),
                message: "Note with that title already exists".to_string(),
            };
            return Err((StatusCode::CONFLICT, Json(e)));
        }
        let e = ErrorResponse {
            error: format!("Database error: {}", e.to_string()),
            message: "Error when adding".to_string(),
        };
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e)));
    }

    let result = SuccessResponse {
        data: "Book created successfully".to_string(),
        message: "Success".to_string(),
    };
    Ok((StatusCode::CREATED, Json(result)))
}
