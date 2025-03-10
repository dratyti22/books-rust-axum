use crate::common::{login_user_token_get, run_test};
use assert2::check;
use cookie::Cookie;
use serde_json::json;

#[test]
fn test_register_user() {
    run_test(|server| {
        Box::pin(async move {
            let register_payload = json!({
                "first_name": "Admin",
                "last_name": "User",
                "middle_name": "Test",
                "age": 30,
                "email": "admin@example.com",
                "password": "password123"
            });

            let response = server
                .post("/api/v1/user/register/")
                .json(&register_payload)
                .await;

            assert_eq!(response.status_code().as_u16(), 201);
        })
    });
}

#[test]
fn test_login_user() {
    run_test(|server| {
        Box::pin(async move {
            let register_payload = json!({
                "first_name": "Admin2",
                "last_name": "User2",
                "middle_name": "Test2",
                "age": 31,
                "email": "admin2@example.com",
                "password": "password1234"
            });

            server
                .post("/api/v1/user/register/")
                .json(&register_payload)
                .await;

            let login_payload = json!({
                "email": "admin2@example.com",
                "password": "password1234"
            });

            let login_response = server
                .post("/api/v1/user/login/")
                .json(&login_payload)
                .await;

            check!(login_response.status_code().as_u16() == 200);
        })
    })
}

#[test]
fn test_logout_user() {
    run_test(|server| {
        Box::pin(async move {
            let (_, token, refresh) = login_user_token_get(&server).await;
            let cookie = Cookie::new("refresh_token", refresh);
            let response = server
                .post("/api/v1/user/logout/")
                .authorization(format!("Bearer {}", token))
                .add_cookie(cookie)
                .await;

            assert_eq!(response.status_code().as_u16(), 200);
        })
    })
}
