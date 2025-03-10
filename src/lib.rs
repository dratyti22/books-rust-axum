use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE};
use axum::http::{HeaderValue, Method};
use dotenv::dotenv;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
mod settings;
use crate::route::init_router;
pub use settings::Settings;

mod api_doc;
mod books;
mod middleware;
pub mod route;
mod service;
mod users;

#[derive(Debug, Clone)]
pub struct AppState {
    db: Pool<Postgres>,
    env: Settings,
    redis: Client,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, env: Settings, redis: Client) -> Self {
        AppState { db, env, redis }
    }
    pub fn db(&self) -> &Pool<Postgres> {
        &self.db
    }

    pub fn env(&self) -> &Settings {
        &self.env
    }

    pub fn redis(&self) -> &Client {
        &self.redis
    }
}

pub async fn start_server() {
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    info!("🚀 Server started at {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
