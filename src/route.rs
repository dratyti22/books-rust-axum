use std::sync::Arc;
use axum::Router;
use crate::AppState;

pub fn init_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .with_state(app_state)
}
