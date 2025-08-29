use crate::{config::AppConfig, http::routes::routes::create_routes};
use tokio::{net::TcpListener, sync::watch, task::JoinHandle};
use tracing::info;

pub async fn start_http_server(
    config: AppConfig,
    shutdown_rx: watch::Receiver<()>,
) -> Result<
    JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let http_addr = format!("{}:{}", config.http.address, config.http.port);
    let listener = tokio::net::TcpListener::bind(http_addr.clone()).await?;
    info!("🚀 Starting BPP-WEBHOOK server on {:?}", http_addr);

    let http_server = tokio::spawn(run_http_server(listener, shutdown_rx, config.clone()));

    Ok(http_server)
}

pub async fn run_http_server(
    listener: TcpListener,
    mut shutdown_rx: watch::Receiver<()>,
    config: AppConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_routes(config);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown_rx.changed().await.ok();
        })
        .await?;

    Ok(())
}
