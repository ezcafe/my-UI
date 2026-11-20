use miniflux_client::client::MinifluxClient;
use miniflux_client::schema::build_schema;
use server::app::build_app;
use server::config::Config;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting my-ui wrapper server...");
    let config = Config::from_env()?;
    let config = Arc::new(config);

    let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
    let schema = build_schema(client);
    let app = build_app(config.clone(), schema);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
