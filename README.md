Static http server

# Usage

Currently, only the simplest http request is resolved and returned, and only the specified folder can be routed through the Future of the route, so as to query the current access path file.
No other mechanism exists

main
```rust
use http_server::StaticHttp;
use config::Config;
#[tokio::main]
async fn main() {
    let config = Config::read_config().unwrap();

    StaticHttp::new()
    .bind(&config.ip_port).await.unwrap()
    .route(&config.web_page).unwrap()
    .run(true).await.unwrap();
}
```

run
```shell
cargo run /var/www
```
