use miniflux_client::client::MinifluxClient;
use miniflux_client::schema::build_schema;
use server::app::build_app;
use server::config::{mask_database_url, Config};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Config::load_dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,server=info,ai=info".into()),
        )
        .init();

    println!("Starting my-ui wrapper server...");
    let config = Config::from_env()?;
    let config = Arc::new(config);

    println!(
        "[SERVER_INIT] auth_enabled={} ai_enabled={} llm_cmd='{}' llm_model='{}' extra_args={:?} database_url='{}'",
        config.auth_enabled,
        config.ai_enabled,
        config.llm_cli_cmd,
        config.llm_model,
        config.llm_extra_args,
        mask_database_url(&config.database_url)
    );

    let client = MinifluxClient::new(
        config.miniflux_base_url.clone(),
        config.miniflux_api_key.clone(),
    );
    let schema = build_schema(client);
    let app = build_app(config.clone(), schema).await;

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
