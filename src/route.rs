use std::sync::Arc;
use axum::Router;
use crate::AppState;
use crate::users::route::auth_routes;

pub fn init_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/auth", auth_routes(app_state.clone()))
        .with_state(app_state)
}
