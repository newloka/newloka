//! Sync and mesh networking
//!
//! Peer discovery, delta sync, vector clock handling,
//! deterministic conflict detection, and secure record transfer.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Node manifest broadcast during peer discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeManifest {
    pub node_id: String,
    pub display_name: String,
    pub tier: crate::DeploymentTier,
    pub public_key: Vec<u8>,
    pub last_sync_timestamp: i64,
    pub supported_resource_types: Vec<String>,
    pub mesh_enabled: bool,
}

/// Delta sync request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaSyncRequest {
    pub from_node: String,
    pub to_node: String,
    pub since_timestamp: i64,
    pub known_vector_clocks: Vec<(String, Vec<(u32, u64)>)>,
}

/// Delta sync response containing encrypted records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaSyncResponse {
    pub from_node: String,
    pub records: Vec<SyncRecord>,
    pub conflicts: Vec<SyncConflict>,
    pub timestamp: i64,
}

/// A record packaged for sync transfer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncRecord {
    pub id: String,
    pub resource_type: String,
    pub encrypted_payload: Vec<u8>,
    pub vector_clock: Vec<(u32, u64)>,
    pub modified_at: i64,
    pub hash: String,
    pub deleted: bool,
    pub patient_id: Option<String>,
    pub department_id: Option<String>,
}

/// Conflict that requires explicit resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncConflict {
    pub resource_id: String,
    pub resource_type: String,
    pub local_clock: Vec<(u32, u64)>,
    pub remote_clock: Vec<(u32, u64)>,
    pub conflict_type: ConflictType,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    ConcurrentEdit,
    MedicationChange,
    DeleteVsUpdate,
    DepartmentMismatch,
    SensitivityEscalation,
}

/// Sync engine for CRDT-based mesh synchronization.
pub struct SyncEngine {
    node_id: String,
    local_clock: Vec<(u32, u64)>,
}

impl SyncEngine {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            local_clock: vec![],
        }
    }

    /// Increment the local vector clock for a write.
    pub fn tick(&mut self) -> Vec<(u32, u64)> {
        let node_hash = self.hash_node_id();
        let mut found = false;
        for (id, count) in &mut self.local_clock {
            if *id == node_hash {
                *count += 1;
                found = true;
                break;
            }
        }
        if !found {
            self.local_clock.push((node_hash, 1));
        }
        self.local_clock.clone()
    }

    fn hash_node_id(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.node_id.hash(&mut hasher);
        let h = hasher.finish();
        (h & 0xffffffff) as u32
    }

    /// Merge two vector clocks, returning the merged result.
    pub fn merge_clocks(a: &[(u32, u64)], b: &[(u32, u64)]) -> Vec<(u32, u64)> {
        let mut merged: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
        for (id, count) in a {
            merged.insert(*id, *count);
        }
        for (id, count) in b {
            merged
                .entry(*id)
                .and_modify(|v| *v = (*v).max(*count))
                .or_insert(*count);
        }
        let mut result: Vec<(u32, u64)> = merged.into_iter().collect();
        result.sort_by_key(|(id, _)| *id);
        result
    }

    /// Determine if one clock dominates another (happens-before).
    pub fn dominates(a: &[(u32, u64)], b: &[(u32, u64)]) -> bool {
        let merged = Self::merge_clocks(a, b);
        a == merged.as_slice() && a != b
    }

    /// Detect conflict between two clocks (concurrent edits).
    pub fn is_concurrent(a: &[(u32, u64)], b: &[(u32, u64)]) -> bool {
        !Self::dominates(a, b) && !Self::dominates(b, a) && a != b
    }

    /// Build a sync manifest from local storage changes.
    pub async fn build_manifest(
        &self,
        storage: &crate::storage::StorageEngine,
        since: i64,
    ) -> crate::Result<DeltaSyncResponse> {
        let records = storage.changed_since(since).await?;
        let mut sync_records = Vec::with_capacity(records.len());
        let conflicts = vec![];

        for row in records {
            sync_records.push(SyncRecord {
                id: row.id.clone(),
                resource_type: row.resource_type.clone(),
                encrypted_payload: row.content,
                vector_clock: serde_json::from_str(&row.vector_clock)?,
                modified_at: row.modified_at,
                hash: row.hash,
                deleted: row.deleted,
                patient_id: row.patient_id,
                department_id: row.department_id,
            });
        }

        Ok(DeltaSyncResponse {
            from_node: self.node_id.clone(),
            records: sync_records,
            conflicts,
            timestamp: Utc::now().timestamp_millis(),
        })
    }

    /// Evaluate conflicts for a received sync batch.
    pub fn evaluate_conflicts(
        &self,
        incoming: &[SyncRecord],
        local_records: &[crate::storage::EncryptedRecord],
    ) -> Vec<SyncConflict> {
        let mut conflicts = vec![];
        let local_map: std::collections::HashMap<String, &crate::storage::EncryptedRecord> =
            local_records.iter().map(|r| (r.id.clone(), r)).collect();

        for rec in incoming {
            if let Some(local) = local_map.get(&rec.id) {
                let local_vc: Vec<(u32, u64)> =
                    serde_json::from_str(&local.vector_clock).unwrap_or_default();
                if Self::is_concurrent(&rec.vector_clock, &local_vc) {
                    let conflict_type = if rec.resource_type == "MedicationRequest" {
                        ConflictType::MedicationChange
                    } else if rec.deleted != local.deleted {
                        ConflictType::DeleteVsUpdate
                    } else {
                        ConflictType::ConcurrentEdit
                    };
                    conflicts.push(SyncConflict {
                        resource_id: rec.id.clone(),
                        resource_type: rec.resource_type.clone(),
                        local_clock: local_vc,
                        remote_clock: rec.vector_clock.clone(),
                        conflict_type,
                        details: format!(
                            "Concurrent modification detected on {}:{}",
                            rec.resource_type, rec.id
                        ),
                    });
                }
            }
        }
        conflicts
    }
}
