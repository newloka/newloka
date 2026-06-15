//! CRDT-based encrypted storage
//!
//! Local-first SQLite storage with CRDT merge support.
//! All records encrypted at rest with per-record keys wrapped by DMK.

use chrono::Utc;
use serde_json;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

/// Storage backend using SQLite with CRDT metadata.
pub struct StorageEngine {
    pool: Pool<Sqlite>,
    node_id: String,
    dmk: crate::crypto::DeviceMasterKey,
}

/// Encrypted record row as stored in SQLite.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EncryptedRecord {
    pub id: String,
    pub resource_type: String,
    pub content: Vec<u8>,
    pub nonce: Vec<u8>,
    pub vector_clock: String,
    pub created_at: i64,
    pub modified_at: i64,
    pub hash: String,
    pub deleted: bool,
    pub patient_id: Option<String>,
    pub department_id: Option<String>,
}

/// Audit event row for persistent audit trail.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditEventRow {
    pub id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub actor_id: String,
    pub node_id: String,
    pub patient_id: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: String,
    pub outcome: String,
    pub details: Option<String>,
    pub previous_hash: Option<String>,
    pub entry_hash: String,
    pub signature: Vec<u8>,
}

impl StorageEngine {
    /// Open (or create) the SQLite backing store.
    pub async fn open(
        db_path: &str,
        node_id: String,
        dmk: crate::crypto::DeviceMasterKey,
    ) -> crate::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .min_connections(1)
            .max_connections(5)
            .connect(db_path)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fhir_resources (
                id TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                content BLOB NOT NULL,
                nonce BLOB NOT NULL,
                vector_clock TEXT NOT NULL DEFAULT '[]',
                created_at INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                hash TEXT NOT NULL,
                deleted INTEGER NOT NULL DEFAULT 0,
                patient_id TEXT,
                department_id TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_type ON fhir_resources(resource_type);
            CREATE INDEX IF NOT EXISTS idx_patient ON fhir_resources(patient_id);
            CREATE INDEX IF NOT EXISTS idx_department ON fhir_resources(department_id);
            CREATE INDEX IF NOT EXISTS idx_modified ON fhir_resources(modified_at);
            CREATE INDEX IF NOT EXISTS idx_hash ON fhir_resources(hash);

            CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                patient_id TEXT,
                resource_id TEXT,
                resource_type TEXT,
                action TEXT NOT NULL,
                outcome TEXT NOT NULL,
                details TEXT,
                previous_hash TEXT,
                entry_hash TEXT NOT NULL,
                signature BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_events(actor_id);
            CREATE INDEX IF NOT EXISTS idx_audit_patient ON audit_events(patient_id);
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool, node_id, dmk })
    }

    /// Check if this is a fresh database.
    pub async fn is_empty(&self) -> crate::Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fhir_resources")
            .fetch_one(&self.pool)
            .await?;
        Ok(count == 0)
    }

    /// Hash the node id into a deterministic `u32` for vector clocks.
    fn node_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.node_id.hash(&mut hasher);
        let h = hasher.finish();
        (h & 0xffffffff) as u32
    }

    /// Compute the next vector clock for `id`, incrementing this node's counter.
    async fn next_vector_clock(&self, id: &str) -> crate::Result<String> {
        let existing: Option<String> =
            sqlx::query_scalar("SELECT vector_clock FROM fhir_resources WHERE id = ?1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        let mut vc: Vec<(u32, u64)> = if let Some(s) = existing {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            vec![]
        };

        let node_hash = self.node_hash();
        let mut found = false;
        for (nid, count) in &mut vc {
            if *nid == node_hash {
                *count += 1;
                found = true;
                break;
            }
        }
        if !found {
            vc.push((node_hash, 1));
        }

        Ok(serde_json::to_string(&vc)?)
    }

    /// Store raw JSON resource (from HTTP layer).
    ///
    /// Uses `INSERT OR REPLACE`. On update, preserves the original `created_at`
    /// and increments the per-node vector clock.
    pub async fn store_json(
        &self,
        resource_type: &str,
        id: &str,
        json: &serde_json::Value,
        patient_id: Option<String>,
        department_id: Option<String>,
    ) -> crate::Result<String> {
        let json_bytes = serde_json::to_vec(json)?;
        let hash = crate::crypto::hash_resource(&json_bytes);

        let prk = crate::crypto::PatientRecordKey::generate();
        let (ciphertext, nonce) = prk.encrypt(&json_bytes)?;
        let (wrapped_prk, wrap_nonce) = crate::crypto::wrap_prk(&self.dmk, &prk)?;

        let mut content = Vec::with_capacity(
            wrapped_prk.len() + wrap_nonce.len() + nonce.len() + ciphertext.len() + 4,
        );
        content.extend_from_slice(&(wrapped_prk.len() as u32).to_be_bytes());
        content.extend_from_slice(&wrapped_prk);
        content.extend_from_slice(&wrap_nonce);
        content.extend_from_slice(&nonce);
        content.extend_from_slice(&ciphertext);

        let now = Utc::now().timestamp_millis();
        let vc = self.next_vector_clock(id).await?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO fhir_resources
            (id, resource_type, content, nonce, vector_clock, created_at, modified_at, hash, deleted, patient_id, department_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(id)
        .bind(resource_type)
        .bind(&content)
        .bind(&nonce[..])
        .bind(&vc)
        .bind(now)
        .bind(now)
        .bind(&hash)
        .bind(false)
        .bind(&patient_id)
        .bind(&department_id)
        .execute(&self.pool)
        .await?;

        Ok(id.to_string())
    }

    /// Convenience: store a FHIR resource directly.
    pub async fn store(
        &self,
        resource: &crate::fhir::FhirResource,
        patient_id: Option<String>,
        department_id: Option<String>,
    ) -> crate::Result<String> {
        let json = serde_json::to_value(resource)?;
        self.store_json(
            resource.resource_type(),
            resource.id(),
            &json,
            patient_id,
            department_id,
        )
        .await
    }

    /// Retrieve decrypted JSON resource.
    pub async fn get_json(&self, id: &str) -> crate::Result<Option<serde_json::Value>> {
        let row: Option<EncryptedRecord> =
            sqlx::query_as("SELECT * FROM fhir_resources WHERE id = ?1 AND deleted = 0")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let json = self.decrypt_json(&row).await?;
        Ok(Some(json))
    }

    /// Convenience: retrieve and deserialize a FHIR resource.
    pub async fn get(&self, id: &str) -> crate::Result<Option<crate::fhir::FhirResource>> {
        match self.get_json(id).await? {
            Some(json) => {
                let resource: crate::fhir::FhirResource = serde_json::from_value(json)?;
                Ok(Some(resource))
            }
            None => Ok(None),
        }
    }

    async fn decrypt_json(&self, row: &EncryptedRecord) -> crate::Result<serde_json::Value> {
        let content = &row.content;
        let wrapped_len =
            u32::from_be_bytes([content[0], content[1], content[2], content[3]]) as usize;
        let offset = 4;
        let wrapped_prk = &content[offset..offset + wrapped_len];
        let wrap_nonce = &content[offset + wrapped_len..offset + wrapped_len + 12];
        let nonce = &content[offset + wrapped_len + 12..offset + wrapped_len + 24];
        let ciphertext = &content[offset + wrapped_len + 24..];

        let wrap_nonce_arr: [u8; 12] = wrap_nonce
            .try_into()
            .map_err(|_| crate::NewLokaError::Crypto("bad wrap nonce".into()))?;
        let prk = crate::crypto::unwrap_prk(&self.dmk, wrapped_prk, &wrap_nonce_arr)?;

        let nonce_arr: [u8; 12] = nonce
            .try_into()
            .map_err(|_| crate::NewLokaError::Crypto("bad nonce".into()))?;
        let plaintext = prk.decrypt(ciphertext, &nonce_arr)?;
        let json: serde_json::Value = serde_json::from_slice(&plaintext)?;
        Ok(json)
    }

    /// Search resources by type and optional patient.
    pub async fn search_json(
        &self,
        resource_type: &str,
        patient_id: Option<&str>,
    ) -> crate::Result<Vec<serde_json::Value>> {
        let rows: Vec<EncryptedRecord> = if let Some(pid) = patient_id {
            sqlx::query_as(
                "SELECT * FROM fhir_resources WHERE resource_type = ?1 AND patient_id = ?2 AND deleted = 0 ORDER BY modified_at DESC LIMIT 500"
            )
            .bind(resource_type)
            .bind(pid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT * FROM fhir_resources WHERE resource_type = ?1 AND deleted = 0 ORDER BY modified_at DESC LIMIT 500"
            )
            .bind(resource_type)
            .fetch_all(&self.pool)
            .await?
        };

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(self.decrypt_json(&row).await?);
        }
        Ok(results)
    }

    /// Convenience: search and deserialize FHIR resources.
    pub async fn search(
        &self,
        resource_type: &str,
        patient_id: Option<&str>,
    ) -> crate::Result<Vec<crate::fhir::FhirResource>> {
        let json_results = self.search_json(resource_type, patient_id).await?;
        let mut resources = Vec::with_capacity(json_results.len());
        for json in json_results {
            resources.push(serde_json::from_value(json)?);
        }
        Ok(resources)
    }

    /// Soft delete a resource.
    pub async fn soft_delete(&self, id: &str) -> crate::Result<bool> {
        let now = Utc::now().timestamp_millis();
        let result =
            sqlx::query("UPDATE fhir_resources SET deleted = 1, modified_at = ?1 WHERE id = ?2")
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Get all modified records since a timestamp (for sync).
    pub async fn changed_since(&self, since: i64) -> crate::Result<Vec<EncryptedRecord>> {
        let rows: Vec<EncryptedRecord> =
            sqlx::query_as("SELECT * FROM fhir_resources WHERE modified_at > ?1")
                .bind(since)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }

    /// Store an audit event persistently.
    pub async fn store_audit(&self, entry: &crate::audit::AuditEntry) -> crate::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_events
            (id, timestamp, event_type, actor_id, node_id, patient_id, resource_id, resource_type, action, outcome, details, previous_hash, entry_hash, signature)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
        )
        .bind(&entry.id)
        .bind(entry.timestamp.timestamp_millis())
        .bind(format!("{:?}", entry.event_type))
        .bind(&entry.actor_id)
        .bind(&entry.node_id)
        .bind(&entry.patient_id)
        .bind(&entry.resource_id)
        .bind(&entry.resource_type)
        .bind(&entry.action)
        .bind(format!("{:?}", entry.outcome))
        .bind(&entry.details)
        .bind(&entry.previous_hash)
        .bind(&entry.entry_hash)
        .bind(&entry.signature)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Retrieve audit events.
    pub async fn search_audit(&self, limit: i64) -> crate::Result<Vec<AuditEventRow>> {
        let rows: Vec<AuditEventRow> =
            sqlx::query_as("SELECT * FROM audit_events ORDER BY timestamp DESC LIMIT ?1")
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }

    /// Count resources by type.
    pub async fn count_by_type(&self, resource_type: &str) -> crate::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM fhir_resources WHERE resource_type = ?1 AND deleted = 0",
        )
        .bind(resource_type)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }
}
