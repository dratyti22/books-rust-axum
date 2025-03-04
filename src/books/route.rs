use crate::books::book_handler::create_book;
use crate::books::genres_handler::create_genres;
use crate::middleware::jwt_auth::{auth_admin, auth_author_worker_admin};
use crate::AppState;
use axum::routing::post;
use axum::{middleware, Router};
use std::sync::Arc;

pub fn books_routers(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/genres/create/",
            post(create_genres).route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_admin,
            )),
        )
        .route(
            "/create/",
            post(create_book).route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_author_worker_admin,
            )),
        )
        .with_state(app_state)
}
