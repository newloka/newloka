use newloka_storage::{EncryptedRecord, StorageEngine};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "fixtures/"]
pub struct DemoFixtures;

pub const DEMO_MASTER_KEY_BYTES: [u8; 32] = *b"DEMOMASTERKEY1234567890123456789";

pub async fn seed_demo_data(
    storage: &StorageEngine,
    node_id: &str,
) -> anyhow::Result<()> {
    tracing::info!("Importing pre-encrypted demo dataset fixtures...");

    // 1. Load manifest
    let manifest_file = DemoFixtures::get("manifest.json")
        .ok_or_else(|| anyhow::anyhow!("Missing demo manifest.json in embedded fixtures"))?;
    let manifest: Vec<serde_json::Value> = serde_json::from_slice(&manifest_file.data)?;

    let mut first_check_done = false;

    for item in &manifest {
        let filename = item["file"].as_str().unwrap_or("");
        let path = format!("records/{}", filename);
        let record_file = DemoFixtures::get(&path)
            .ok_or_else(|| anyhow::anyhow!("Missing encrypted fixture file: {}", path))?;
        let record: EncryptedRecord = serde_json::from_slice(&record_file.data)?;

        // Cryptographic check: Verify that this storage engine's DMK can actually decrypt the record.
        // If someone passed a real user key or random key, decryption will fail at the AEAD tag.
        if !first_check_done {
            if let Err(e) = storage.decrypt_record(&record).await {
                anyhow::bail!(
                    "Cryptographic key mismatch: StorageEngine cannot decrypt demo fixtures. (Expected Demo Master Key). Error: {}",
                    e
                );
            }
            first_check_done = true;
        }

        storage.store_encrypted_record(&record).await?;
    }

    // 2. Generate signed demo audit trail
    let signer = newloka_core::crypto::AuditSigner::generate();
    let mut audit = newloka_core::audit::AuditEngine::new(node_id.to_string(), signer);
    for i in 0..10 {
        let entry = audit.log(
            newloka_core::audit::AuditEventType::Access,
            format!("user-{}", i),
            None,
            None,
            Some("Patient".to_string()),
            if i % 3 == 0 { "read".to_string() } else { "create".to_string() },
            newloka_core::audit::AuditOutcome::Success,
            Some(format!("Seeded audit entry {}", i)),
        )?;
        storage.store_audit(&entry).await?;
    }

    // 3. Seed demo users
    seed_demo_users(storage).await?;

    tracing::info!("Successfully ingested {} pre-encrypted demo records into StorageEngine", manifest.len());
    Ok(())
}

pub async fn seed_demo_users(storage: &StorageEngine) -> anyhow::Result<()> {
    let existing = storage.get_user_by_username("admin").await.ok().flatten();
    if existing.is_some() {
        return Ok(());
    }
    let users = vec![
        (
            "admin",
            "New Loka Admin",
            vec!["administrator"],
            None,
            vec![],
            vec![],
        ),
        (
            "pathlab",
            "Pathology Lab",
            vec!["lab_technician"],
            Some("Pathology".to_string()),
            vec!["path-team".to_string()],
            vec!["pathology"],
        ),
        (
            "imaging",
            "Imaging Tech",
            vec!["imaging_technician"],
            Some("Radiology".to_string()),
            vec!["radio-team".to_string()],
            vec!["radiology"],
        ),
        (
            "resident",
            "Resident Doctor",
            vec!["resident_doctor"],
            Some("Internal Medicine".to_string()),
            vec!["team-alpha".to_string()],
            vec![],
        ),
        (
            "depthead",
            "Department Head",
            vec!["department_head"],
            Some("Internal Medicine".to_string()),
            vec!["team-alpha".to_string(), "mgmt-team".to_string()],
            vec![],
        ),
    ];
    for (username, display_name, roles, dept, teams, labs) in users {
        let (salt, hash) = newloka_core::identity::Authenticator::hash_password(username)?;
        let parsed_roles: Vec<newloka_core::identity::Role> =
            roles.iter().filter_map(|r| r.parse().ok()).collect();
        let parsed_labs: Vec<newloka_core::cpoe::LabDepartment> =
            labs.iter().filter_map(|l| l.parse().ok()).collect();
        let user = newloka_core::identity::User {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            roles: parsed_roles,
            department_id: dept,
            team_ids: teams,
            lab_affiliations: parsed_labs,
            active: true,
            created_at: chrono::Utc::now(),
            password_hash: hash,
            salt,
            totp_secret: None,
            last_login: None,
        };
        storage.store_user(&user).await?;
    }
    tracing::info!("Seeded demo users");
    Ok(())
}
