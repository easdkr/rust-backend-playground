use redis::aio::ConnectionManager;

pub async fn connect_manager(url: &str) -> Result<ConnectionManager, String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    ConnectionManager::new(client)
        .await
        .map_err(|e| e.to_string())
}
