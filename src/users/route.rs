use std::sync::Arc;
use axum::routing::post;
use axum::Router;
use crate::AppState;
use crate::users::handler::register_user_handler;

pub fn auth_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/register/", post(register_user_handler))
        .with_state(app_state)
}
