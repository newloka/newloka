//! Application state and node configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub node_id: String,
    pub storage: Arc<newloka_core::storage::StorageEngine>,
    pub config: NodeConfig,
    pub sessions: Arc<RwLock<HashMap<String, newloka_core::identity::Session>>>,
}

impl AppState {
    pub fn new(
        node_id: String,
        storage: Arc<newloka_core::storage::StorageEngine>,
        config: NodeConfig,
    ) -> Self {
        Self {
            node_id,
            storage,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeConfig {
    pub tier: String,
    pub node_id: String,
    pub department: String,
    pub sync_enabled: bool,
    pub mesh_enabled: bool,
    pub offline_auth: String,
    pub language: String,
    pub emergency_access: bool,
    pub page_size: usize,
    pub default_encounter_status: String,
    pub offline_queue_auto_flush: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            tier: "T1".to_string(),
            node_id: "default-node".to_string(),
            department: "default".to_string(),
            sync_enabled: true,
            mesh_enabled: false,
            offline_auth: "pin".to_string(),
            language: "en".to_string(),
            emergency_access: false,
            page_size: 20,
            default_encounter_status: "in-progress".to_string(),
            offline_queue_auto_flush: true,
        }
    }
}
