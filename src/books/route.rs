use crate::AppState;
use crate::books::book_handler::{
    create_book, delete_book, get_all_books, get_one_book, update_book,
};
use crate::books::genres_handler::{create_genres, get_all_genres};
use crate::middleware::jwt_auth::{auth_admin, auth_author_worker_admin};
use axum::routing::{delete, get, patch, post};
use axum::{Router, middleware};
use std::sync::Arc;

pub fn genre_routers(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/create/",
            post(create_genres).route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_admin,
            )),
        )
        .route("/", get(get_all_genres))
        .with_state(app_state)
}

pub fn books_routers(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/genres", genre_routers(app_state.clone()))
        .route("/", get(get_all_books))
        .route("/{id}", get(get_one_book))
        .route(
            "/create/",
            post(create_book).route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_author_worker_admin,
            )),
        )
        .route(
            "/delete/{id}/",
            delete(delete_book).route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_author_worker_admin,
            )),
        )
        .route(
            "/update/{id}/",
            patch(update_book).route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_author_worker_admin,
            )),
        )
        .with_state(app_state)
}
