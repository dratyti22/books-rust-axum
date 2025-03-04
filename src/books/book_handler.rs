use crate::books::model::Books;
use crate::books::response::BookResponse;
use crate::books::schema::{BookSchema, BookUpdateSchema};
use crate::middleware::jwt_auth::JWTAuthMiddleware;
use crate::service::response_server::{APIResult, ErrorResponse, SuccessResponse};
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum_macros::debug_handler;
use chrono::Datelike;
use std::sync::Arc;
use validator::Validate;

#[debug_handler]
pub async fn create_book(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<BookSchema>,
) -> APIResult<String> {
    if body.validate().is_err() {
        let error = ErrorResponse {
            error: "Invalid".to_string(),
            message: "Invalid input data".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error)));
    }

    let user_id = user.user.id;
    let publication_year = chrono::Utc::now().year() as i16;
    let query_result = sqlx::query(
        r#"INSERT INTO books (title, description, author_id, genre_id, isbn, cover_image, price, discount, publication_year)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"#,
    )
        .bind(body.title.to_owned())
        .bind(body.description.to_owned())
        .bind(user_id.to_owned())
        .bind(body.genre_id.to_owned())
        .bind(body.isbn.to_owned())
        .bind(body.cover_image.to_owned())
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

pub async fn delete_book(
    State(data): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> APIResult<String> {
    let query_result = sqlx::query!(r#"DELETE FROM books WHERE id = $1"#, id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            let e = ErrorResponse {
                error: format!("Database error: {}", e.to_string()),
                message: "Error when deleting".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(e))
        })?;

    if query_result.rows_affected() == 0 {
        let e = ErrorResponse {
            error: "".to_string(),
            message: "Book not found".to_string(),
        };
        return Err((StatusCode::NOT_FOUND, Json(e)));
    }

    let response = SuccessResponse {
        data: "Book deleted successfully".to_string(),
        message: "Success".to_string(),
    };
    Ok((StatusCode::NO_CONTENT, Json(response)))
}

pub async fn update_book(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<BookUpdateSchema>,
) -> APIResult<Books> {
    if body.validate().is_err() {
        let e = ErrorResponse {
            error: "Invalid".to_string(),
            message: "Invalid input data".to_string(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(e)));
    }

    sqlx::query_as!(
        Books,
        r#"
        UPDATE books
        SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            cover_image = COALESCE($3, cover_image),
            price = COALESCE($4, price),
            discount = COALESCE($5, discount)
        WHERE id = $6
        RETURNING *
        "#,
        body.title,
        body.description,
        body.cover_image,
        body.price,
        body.discount,
        id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            let e = ErrorResponse {
                error: "".to_string(),
                message: "Book not found".to_string(),
            };
            (StatusCode::NOT_FOUND, Json(e))
        }
        _ => {
            let e = ErrorResponse {
                error: format!("Database error: {}", e.to_string()),
                message: "Error when updating book in database".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(e))
        }
    })?;

    let updated_book = sqlx::query_as!(Books, "SELECT * FROM books WHERE id = $1", id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            let e = ErrorResponse {
                error: format!("Database error: {}", e.to_string()),
                message: "Error when fetching updated book from database".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(e))
        })?;

    let response = SuccessResponse {
        data: updated_book,
        message: "Book updated successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_all_books(State(data): State<Arc<AppState>>) -> APIResult<Vec<BookResponse>> {
    let books_response = sqlx::query_as!(Books, "SELECT * FROM books")
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            let e = ErrorResponse {
                error: format!("Database error: {}", e.to_string()),
                message: "Error when fetching all books from database".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(e))
        })?;

    let response_books: Vec<BookResponse> = books_response
        .into_iter()
        .map(BookResponse::from_book)
        .collect();

    let response = SuccessResponse {
        data: response_books,
        message: "Books fetched successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_one_book(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> APIResult<BookResponse> {
    let query = sqlx::query_as!(Books, r#"SELECT * FROM books WHERE id = $1"#, id)
        .fetch_one(&data.db)
        .await;

    match query {
        Ok(book) => {
            let response = SuccessResponse {
                data: BookResponse::from_book(book),
                message: "Book fetched successfully".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(sqlx::Error::RowNotFound) => {
            let e = ErrorResponse {
                error: "".to_string(),
                message: "Book not found".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(e)))
        }
        Err(e) => {
            let e = ErrorResponse {
                error: format!("Database error: {}", e.to_string()),
                message: "Error when fetching book from database".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e)))
        }
    }
}
