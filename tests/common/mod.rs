use axum_test::TestServer;
use books::Settings;
use books::{AppState, route::init_router};
use redis::Client;
use serde_json::json;
use sqlx::{PgPool, Pool, Postgres};
use std::pin::Pin;
use std::sync::Arc;

pub const TEST_DB_NAME: &str = "book_rust_test";

pub fn run_test<T>(test: T) -> ()
where
    T: std::panic::UnwindSafe,
    T: FnOnce(TestServer) -> Pin<Box<dyn Future<Output = ()> + 'static>>,
{
    let result = std::panic::catch_unwind(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let server = init_test_server().await;

                test(server).await;

                drop_test_database().await.unwrap();
            })
    });
    assert!(result.is_ok());
}

pub async fn drop_test_database() -> Result<(), sqlx::Error> {
    dotenv::from_filename(".env.test").ok();
    let master_db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test");

    let pool = PgPool::connect(&master_db_url).await?;

    sqlx::query(
        r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
           FROM pg_stat_activity
           WHERE pg_stat_activity.datname = $1 AND pid <> pg_backend_pid()"#,
    )
    .bind(TEST_DB_NAME)
    .execute(&pool)
    .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{}""#, TEST_DB_NAME))
        .execute(&pool)
        .await?;

    Ok(())
}
async fn create_db(url: &str) -> Result<(), sqlx::Error> {
    let pool = PgPool::connect(url).await?;
    sqlx::query(
        r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
           FROM pg_stat_activity
           WHERE pg_stat_activity.datname = $1 AND pid <> pg_backend_pid()"#,
    )
    .bind(TEST_DB_NAME)
    .execute(&pool)
    .await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{}""#, TEST_DB_NAME))
        .execute(&pool)
        .await?;
    sqlx::query(&format!(r#"CREATE DATABASE "{}""#, TEST_DB_NAME))
        .execute(&pool)
        .await?;
    Ok(())
}

async fn run_migrate(db_url: &str) -> Result<(), String> {
    let output = std::process::Command::new("sqlx")
        .arg("migrate")
        .arg("run")
        .arg("--database-url")
        .arg(db_url)
        .output()
        .map_err(|e| format!("Failed to run migrations: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "Migrations failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

async fn setup_test_a_state() -> Arc<AppState> {
    dotenv::from_filename(".env.test").ok();
    let settings = Settings::init();
    create_db(&settings.database_url).await.unwrap();
    run_migrate(format!("{}{}", &settings.database_url, TEST_DB_NAME).as_str())
        .await
        .unwrap();

    // Подключаемся к тестовой базе данных
    let pool = PgPool::connect(format!("{}{}", &settings.database_url, TEST_DB_NAME).as_str())
        .await
        .expect("Failed to connect to the database");

    // Подключаемся к Redis
    let redis_client = Client::open(&*settings.redis_url).unwrap();

    // Создаём AppState
    Arc::new(AppState::new(pool, settings, redis_client))
}

pub async fn cleanup_db(pool: &Pool<Postgres>) {
    sqlx::query("TRUNCATE TABLE genres, users, books CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean up database");
}

pub async fn init_test_server() -> TestServer {
    drop_test_database().await.ok();
    let app_state = setup_test_a_state().await;
    cleanup_db(&app_state.db()).await;
    let app = init_router(app_state);
    let mut server = TestServer::new(app).expect("Failed to start test server");
    server.save_cookies();
    server
}

pub async fn login_user_token_get(server: &TestServer) -> (String, String, String) {
    // Регистрация пользователя
    let register_payload = json!({
        "first_name": "Admin",
        "last_name": "User",
        "middle_name": "Test",
        "age": 30,
        "email": "admin@example.com",
        "password": "password123"
    });

    let user_response = server
        .post("/api/v1/user/register/")
        .json(&register_payload)
        .await;

    let user: serde_json::Value = user_response.json::<serde_json::Value>();
    let user_id = user
        .get("data")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    println!("User ID: {}", user_id);

    let login_payload = json!({
        "email": "admin@example.com",
        "password": "password123"
    });

    let login_response = server
        .post("/api/v1/user/login/")
        .json(&login_payload)
        .await;

    let cookies: serde_json::Value = login_response.json::<serde_json::Value>();
    let access = cookies
        .get("data")
        .unwrap()
        .get("access_token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let refresh = cookies
        .get("data")
        .unwrap()
        .get("refresh_token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    (user_id, access, refresh)
}
