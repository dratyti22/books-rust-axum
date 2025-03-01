use std::sync::Arc;
use axum::Router;
use crate::AppState;
use crate::books::route::books_routers;
use crate::users::route::user_routes;

pub fn init_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1/user", user_routes(app_state.clone()))
        .nest("/api/v1/book", books_routers(app_state.clone()))
        .with_state(app_state)
}
