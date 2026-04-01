use anyhow::Result;
use tracing::info;
use tokio::net::TcpListener;

use super::RobustState;
use super::router::create_robust_router;

pub async fn start_robust_server(port: u16, state: RobustState) -> Result<()> {
    let router = create_robust_router(state);
    let addr = format!("0.0.0.0:{}", port);

    info!("Starting Robust server on {}", addr);
    info!("  Grid service:        POST /grid");
    info!("  UserAccount service: POST /accounts");
    info!("  Auth service:        POST /auth");
    info!("  Asset service:       POST /assets, GET /assets/{{id}}");
    info!("  Inventory service:   POST /inventory");
    info!("  Presence service:    POST /presence");
    info!("  Avatar service:      POST /avatar");

    let listener = TcpListener::bind(&addr).await
        .map_err(|e| anyhow::anyhow!("Failed to bind Robust server to {}: {}", addr, e))?;

    axum::serve(listener, router).await
        .map_err(|e| anyhow::anyhow!("Robust server error: {}", e))?;

    Ok(())
}
