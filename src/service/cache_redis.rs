use redis::AsyncCommands;
use redis::RedisError;
use serde::de::DeserializeOwned;
use serde::Serialize;
pub async fn get_or_set_cache<T, F, Fut>(
    redis_url: &str,
    key: &str,
    fetch_fn: F,
) -> Result<T, RedisError>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, RedisError>>,
{
    {
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_multiplexed_async_connection().await?;

        if let Ok(cached_value) = conn.get::<_, String>(key).await {
            if let Ok(result) = serde_json::from_str::<T>(&cached_value) {
                return Ok(result);
            }
        }
        let value = fetch_fn().await?;
        if let Ok(serialized) = serde_json::to_string(&value) {
            let _: Result<(), _> = conn.set(key, serialized).await;
        }

        Ok(value)
    }
}
