use crate::AppState;
use crate::api_doc::ApiDoc;
use crate::books::route::books_routers;
use crate::users::route::user_routes;
use axum::Router;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
pub fn init_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1/user", user_routes(app_state.clone()))
        .nest("/api/v1/book", books_routers(app_state.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(app_state)
}
