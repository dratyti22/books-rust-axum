use std::sync::Arc;
use axum::routing::post;
use axum::Router;
use crate::AppState;
use crate::users::handler::{login_user_handler, logout_user_handler, register_user_handler};

pub fn auth_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/register/", post(register_user_handler))
        .route("/login/", post(login_user_handler))
        .route("/logout/", post(logout_user_handler))
        .with_state(app_state)
}
