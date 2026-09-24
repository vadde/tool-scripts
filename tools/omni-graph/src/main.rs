// Omni-Graph Orchestrator Entrypoint
// Architecture: Axum async server connecting to SurrealDB v2 and HuggingFace TEI
// Purpose-built for 100% local Apple Silicon execution

mod analysis;
mod api;
mod condenser;
mod config;
mod db;
mod embedder;
mod ingestion;
mod parser;

use api::{create_router, AppState};
use config::Config;
use db::DbClient;
use embedder::EmbedderClient;
use ingestion::{FileCache, IngestionPipeline};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "omni_graph=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Omni-Graph Semantic Knowledge Hub...");

    let config = Config::from_env();
    info!("Configuration loaded: binding to port {}", config.port);
    info!("SurrealDB endpoint: {}", config.surreal_url);
    info!("HuggingFace TEI endpoint: {}", config.tei_url);

    let db_client = DbClient::new(&config);
    let embedder_client = EmbedderClient::new(config.tei_url.clone());
    let file_cache = FileCache::new();

    let pipeline = Arc::new(IngestionPipeline::new(
        db_client.clone(),
        embedder_client.clone(),
        file_cache,
    ));

    // Wait for SurrealDB to become available and initialize schema (R-002, R-019)
    info!("Waiting for SurrealDB connectivity...");
    let mut retries = 0;
    while !db_client.health().await && retries < 20 {
        sleep(Duration::from_secs(2)).await;
        retries += 1;
        info!("SurrealDB not ready yet, retry {}/20...", retries);
    }

    if retries < 20 {
        info!("SurrealDB connected. Initializing schema...");
        let schema = include_str!("db/schema.surql");
        if let Err(e) = db_client.init_schema(schema).await {
            warn!("Schema initialization notice: {}", e);
        }
    } else {
        warn!("SurrealDB connectivity check timed out; continuing startup in degraded mode.");
    }

    let state = AppState {
        db: db_client,
        embedder: embedder_client,
        pipeline,
    };

    let router = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Omni-Graph REST API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
