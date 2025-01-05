use static_http_erver::StaticHttp;

#[tokio::main]
async fn main() {
    StaticHttp::new()
    .bind("127.0.0.1:80").await.unwrap()
    .route("C:\\Users\\Desktop\\website").unwrap()
    .run(true).await.unwrap();
}
