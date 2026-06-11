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
    #[allow(dead_code)]
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

impl StorageEngine {
    pub async fn open(
        db_path: &str,
        #[allow(dead_code)]
    node_id: String,
        dmk: crate::crypto::DeviceMasterKey,
    ) -> crate::Result<Self> {
        let pool = SqlitePoolOptions::new()
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
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            node_id,
            dmk,
        })
    }

    /// Store a FHIR resource with encryption.
    pub async fn store(
        &self,
        resource: &crate::fhir::FhirResource,
        patient_id: Option<String>,
        department_id: Option<String>,
    ) -> crate::Result<String> {
        let json = serde_json::to_vec(resource)?;
        let hash = crate::crypto::hash_resource(&json);

        // Generate per-record key and encrypt
        let prk = crate::crypto::PatientRecordKey::generate();
        let (ciphertext, nonce) = prk.encrypt(&json)?;
        let (wrapped_prk, wrap_nonce) = crate::crypto::wrap_prk(&self.dmk, &prk)?;

        // Store wrapped key alongside ciphertext
        let mut content = Vec::with_capacity(wrapped_prk.len() + wrap_nonce.len() + nonce.len() + ciphertext.len() + 4);
        content.extend_from_slice(&((wrapped_prk.len() as u32).to_be_bytes()));
        content.extend_from_slice(&wrapped_prk);
        content.extend_from_slice(&wrap_nonce);
        content.extend_from_slice(&nonce);
        content.extend_from_slice(&ciphertext);

        let now = Utc::now().timestamp_millis();
        let vc = serde_json::to_string(&resource.meta().vector_clock)?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO fhir_resources
            (id, resource_type, content, nonce, vector_clock, created_at, modified_at, hash, deleted, patient_id, department_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(resource.id())
        .bind(resource.resource_type())
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

        Ok(resource.id().to_string())
    }

    /// Retrieve and decrypt a FHIR resource.
    pub async fn get(&self,
        id: &str,
    ) -> crate::Result<Option<crate::fhir::FhirResource>> {
        let row: Option<EncryptedRecord> = sqlx::query_as(
            "SELECT * FROM fhir_resources WHERE id = ?1 AND deleted = 0"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else { return Ok(None); };
        let resource = self.decrypt_record(&row).await?;
        Ok(Some(resource))
    }

    async fn decrypt_record(
        &self,
        row: &EncryptedRecord,
    ) -> crate::Result<crate::fhir::FhirResource> {
        let content = &row.content;
        let wrapped_len = u32::from_be_bytes([content[0], content[1], content[2], content[3]]) as usize;
        let offset = 4;
        let wrapped_prk = &content[offset..offset + wrapped_len];
        let wrap_nonce = &content[offset + wrapped_len..offset + wrapped_len + 12];
        let nonce = &content[offset + wrapped_len + 12..offset + wrapped_len + 24];
        let ciphertext = &content[offset + wrapped_len + 24..];

        let wrap_nonce_arr: [u8; 12] = wrap_nonce.try_into().map_err(|_| crate::NewLokaError::Crypto("bad wrap nonce".into()))?;
        let prk = crate::crypto::unwrap_prk(&self.dmk, wrapped_prk, &wrap_nonce_arr)?;

        let nonce_arr: [u8; 12] = nonce.try_into().map_err(|_| crate::NewLokaError::Crypto("bad nonce".into()))?;
        let plaintext = prk.decrypt(ciphertext, &nonce_arr)?;
        let resource: crate::fhir::FhirResource = serde_json::from_slice(&plaintext)?;
        Ok(resource)
    }

    /// Search resources by type and optional patient.
    pub async fn search(
        &self,
        resource_type: &str,
        patient_id: Option<&str>,
    ) -> crate::Result<Vec<crate::fhir::FhirResource>> {
        let rows: Vec<EncryptedRecord> = if let Some(pid) = patient_id {
            sqlx::query_as(
                "SELECT * FROM fhir_resources WHERE resource_type = ?1 AND patient_id = ?2 AND deleted = 0 ORDER BY modified_at DESC"
            )
            .bind(resource_type)
            .bind(pid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT * FROM fhir_resources WHERE resource_type = ?1 AND deleted = 0 ORDER BY modified_at DESC"
            )
            .bind(resource_type)
            .fetch_all(&self.pool)
            .await?
        };

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(self.decrypt_record(&row).await?);
        }
        Ok(results)
    }

    /// Soft delete a resource.
    pub async fn soft_delete(&self,
        id: &str,
        _user_id: &str,
    ) -> crate::Result<bool> {
        let now = Utc::now().timestamp_millis();
        let result = sqlx::query(
            "UPDATE fhir_resources SET deleted = 1, modified_at = ?1 WHERE id = ?2"
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Get all modified records since a timestamp (for sync).
    pub async fn changed_since(
        &self,
        since: i64,
    ) -> crate::Result<Vec<EncryptedRecord>> {
        let rows: Vec<EncryptedRecord> = sqlx::query_as(
            "SELECT * FROM fhir_resources WHERE modified_at > ?1"
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
