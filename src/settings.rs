#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub redis_url: String,
    pub client_origin: String,

    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,

    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_expires_in: String,
    pub refresh_token_max_age: i64,
}

impl Settings {
    pub fn init() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL").unwrap();
        let client_origin = std::env::var("CLIENT_ORIGIN").unwrap();

        let access_token_private_key = std::env::var("ACCESS_TOKEN_PRIVATE_KEY").unwrap();
        let access_token_public_key = std::env::var("ACCESS_TOKEN_PUBLIC_KEY").unwrap();
        let access_token_expires_in = std::env::var("ACCESS_TOKEN_EXPIRED_IN").unwrap();
        let access_token_max_age = std::env::var("ACCESS_TOKEN_MAXAGE").unwrap();

        let refresh_token_private_key = std::env::var("REFRESH_TOKEN_PRIVATE_KEY").unwrap();
        let refresh_token_public_key = std::env::var("REFRESH_TOKEN_PUBLIC_KEY").unwrap();
        let refresh_token_expires_in = std::env::var("REFRESH_TOKEN_EXPIRED_IN").unwrap();
        let refresh_token_max_age = std::env::var("REFRESH_TOKEN_MAXAGE").unwrap();

        Self {
            database_url,
            redis_url,
            client_origin,
            access_token_private_key,
            access_token_public_key,
            refresh_token_private_key,
            refresh_token_public_key,
            access_token_expires_in,
            refresh_token_expires_in,
            access_token_max_age: access_token_max_age.parse::<i64>().unwrap(),
            refresh_token_max_age: refresh_token_max_age.parse::<i64>().unwrap(),
        }
    }
}
