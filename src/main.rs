use config::Config;
use http::HttpServer;

#[tokio::main]
async fn main() {
    let config = Config::read_config().unwrap();

    
    HttpServer::new()
        .bind(&config.ip_port).await.unwrap()
        .route(&config.web_page).unwrap()
        .set_keep_alive(config.keepalive_timeout)
        .run().await;
}