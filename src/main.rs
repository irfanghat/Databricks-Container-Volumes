#![allow(unused)]

mod error;
mod handlers;
mod router;
mod session;
mod state;
mod storage;

use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use dotenvy::dotenv;

use state::AppState;
use storage::VolumeStorage;
use tokio::net::TcpListener;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing::subscriber::set_global_default(FmtSubscriber::new()).unwrap();

    let storage = Arc::new(VolumeStorage::new(
        std::env::var("DATABRICKS_HOST")?,
        std::env::var("DATABRICKS_TOKEN")?,
        std::env::var("DATABRICKS_VOLUME")?,
    ));

    storage.initialize().await?;

    let state = AppState { storage };

    let app = router::router().with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));

    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
