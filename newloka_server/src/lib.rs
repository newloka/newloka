//! New Loka HTTP Server
//!
//! Provides REST API for T1+ deployments.
//! FHIR R4 compatible endpoints with ABAC enforcement.
//!
//! ## Environment Variables
//! - `NEWLOKA_NODE_ID` - Node identity (default: `server-node`)
//! - `NEWLOKA_DB_PATH` - SQLite connection string (default: in-memory)
//! - `NEWLOKA_MASTER_KEY` - Hex-encoded 32-byte device master key (**required** unless `demo` feature is enabled)

use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[cfg(feature = "demo")]
pub use newloka_demo as demo;
pub mod routes;
pub mod rxpad;
pub mod state;
pub mod updater;

pub use crate::routes::auth::{CreateUserRequest, EvaluateAbacRequest, LoginResponse, UserResponse};
pub use crate::routes::pharmacopoeia::{
    pharmacopoeia_categories_handler, pharmacopoeia_check_safety_handler,
    pharmacopoeia_get_entry_handler, pharmacopoeia_get_order_set_handler,
    pharmacopoeia_info_handler, pharmacopoeia_list_order_sets_handler,
    pharmacopoeia_search_handler, PharmacopoeiaSearchQuery, SafetyCheckRequest,
};
pub use crate::state::{AppState, NodeConfig};

/// Resolve the master key from environment or the `demo` feature.
fn resolve_master_key() -> anyhow::Result<newloka_core::crypto::DeviceMasterKey> {
    #[cfg(feature = "demo")]
    {
        tracing::warn!("DEMO FEATURE ENABLED: using hardcoded master key - NOT FOR PRODUCTION");
        Ok(newloka_core::crypto::DeviceMasterKey {
            key: *b"DEMOMASTERKEY1234567890123456789",
        })
    }

    #[cfg(not(feature = "demo"))]
    {
        let hex = std::env::var("NEWLOKA_MASTER_KEY")
            .map_err(|_| anyhow::anyhow!("NEWLOKA_MASTER_KEY environment variable is required in production builds (or enable the `demo` feature)"))?;
        let bytes = newloka_core::crypto::hex_decode(&hex)
            .map_err(|e| anyhow::anyhow!("invalid NEWLOKA_MASTER_KEY hex: {}", e))?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "NEWLOKA_MASTER_KEY must be 64 hex characters (32 bytes), got {}",
                bytes.len()
            ));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(newloka_core::crypto::DeviceMasterKey { key })
    }
}

/// Build the Axum router for the given application state.
pub fn app(state: Arc<RwLock<AppState>>) -> Router {
    routes::create_router()
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the New Loka HTTP server.
pub async fn run(bind: &str) -> anyhow::Result<()> {
    updater::record_start_time();
    let node_id = std::env::var("NEWLOKA_NODE_ID").unwrap_or_else(|_| "server-node".to_string());
    let db_path = std::env::var("NEWLOKA_DB_PATH")
        .unwrap_or_else(|_| "sqlite::memory:?cache=shared".to_string());
    tracing::info!("Using database: {}", db_path);

    let dmk = resolve_master_key()?;
    let storage =
        newloka_core::storage::StorageEngine::open(&db_path, node_id.clone(), dmk).await?;
    #[cfg(feature = "demo")]
    {
        if storage.is_empty().await.unwrap_or(false) {
            tracing::info!("Database empty -- importing encrypted demo dataset fixtures");
            demo::seed_demo_data(&storage, &node_id).await?;
        }
    }
    let storage = std::sync::Arc::new(storage);
    let tier_str = std::env::var("NEWLOKA_TIER").unwrap_or_else(|_| "T1".to_string());
    let is_t0 = tier_str.eq_ignore_ascii_case("T0");
    let config = NodeConfig {
        tier: tier_str.to_uppercase(),
        node_id: node_id.clone(),
        department: std::env::var("NEWLOKA_DEPARTMENT").unwrap_or_else(|_| {
            if is_t0 {
                "Solo Practice".to_string()
            } else {
                "default".to_string()
            }
        }),
        sync_enabled: std::env::var("NEWLOKA_SYNC_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(!is_t0),
        mesh_enabled: std::env::var("NEWLOKA_MESH_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(false),
        offline_auth: std::env::var("NEWLOKA_OFFLINE_AUTH").unwrap_or_else(|_| "pin".to_string()),
        language: std::env::var("NEWLOKA_LANGUAGE").unwrap_or_else(|_| "en".to_string()),
        emergency_access: std::env::var("NEWLOKA_EMERGENCY_ACCESS")
            .unwrap_or_else(|_| "false".to_string())
            == "true",
        page_size: std::env::var("NEWLOKA_PAGE_SIZE")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20),
        default_encounter_status: std::env::var("NEWLOKA_DEFAULT_ENCOUNTER_STATUS")
            .unwrap_or_else(|_| "in-progress".to_string()),
        offline_queue_auto_flush: std::env::var("NEWLOKA_OFFLINE_QUEUE_AUTO_FLUSH")
            .unwrap_or_else(|_| "true".to_string())
            == "true",
    };
    let state = Arc::new(RwLock::new(AppState::new(node_id, storage, config)));

    let server_app = app(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("New Loka server listening on {}", bind);
    axum::serve(listener, server_app).await?;
    Ok(())
}
