mod route;
mod settings;
mod users;
mod middleware;
mod books;

use crate::route::init_router;
use crate::settings::Settings;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE};
use axum::http::{HeaderValue, Method};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use redis::Client;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, info_span};

#[derive(Debug, Clone)]
pub struct AppState {
    db: Pool<Postgres>,
    env: Settings,
    redis: Client,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let settings = Settings::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&settings.database_url)
        .await
    {
        Ok(pool) => {
            info!("Connection to the database is successful!");
            pool
        }
        Err(err) => {
            error!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };
    let redis_client = Client::open(&*settings.redis_url).unwrap();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, COOKIE]);

    let app = init_router(Arc::new(AppState {
        db: pool.clone(),
        env: settings.clone(),
        redis: redis_client.clone(),
    }))
    .layer(cors);

    let listener = tokio::net::TcpListener::bind("localhost:8000")
        .await
        .unwrap();
    info!("🚀 Server started at {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
