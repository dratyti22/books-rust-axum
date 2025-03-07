use crate::middleware::jwt_auth::auth;
use crate::users::handler::{login_user_handler, logout_user_handler, register_user_handler};
use crate::AppState;
use axum::routing::post;
use axum::{middleware, Router};
use std::sync::Arc;

pub fn user_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/register/", post(register_user_handler))
        .route("/login/", post(login_user_handler))
        .route(
            "/logout/",
            post(logout_user_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
