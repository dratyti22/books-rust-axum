use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
    crate::books::genres_handler::get_all_genres,
    crate::books::genres_handler::create_genres,
    crate::books::book_handler::create_book,
    crate::books::book_handler::delete_book,
    crate::books::book_handler::update_book,
    crate::books::book_handler::get_all_books,
    crate::books::book_handler::get_one_book,
    crate::users::handler::register_user_handler,
    crate::users::handler::login_user_handler,
    crate::users::handler::logout_user_handler,
    ),
    tags(
        (name = "Books", description = "API для работы с книгами"),
        (name = "Books genres", description = "API для работы с жанрами у книг"),
        (name = "Users", description = "API для работы с пользователями")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "Bearer",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}
