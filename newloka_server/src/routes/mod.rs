//! Domain Route Modules for New Loka Server
//!
//! Decomposes the monolithic router into modular, single-responsibility domains.

pub mod auth;
pub mod clinical;
pub mod governance;
pub mod ingest;
pub mod pharmacopoeia;
pub mod research;
pub mod resource_helpers;
pub mod rxpad;
pub mod system;

use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

/// Assembles the unified application router from all domain modules.
pub fn create_router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .merge(system::router())
        .merge(auth::router())
        .merge(clinical::router())
        .merge(governance::router())
        .merge(research::router())
        .merge(pharmacopoeia::router())
        .merge(rxpad::router())
        .merge(ingest::router())
}
