use crate::books::route::books_routers;
use crate::users::route::user_routes;
use crate::AppState;
use axum::Router;
use std::sync::Arc;

pub fn init_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1/user", user_routes(app_state.clone()))
        .nest("/api/v1/book", books_routers(app_state.clone()))
        .with_state(app_state)
}
