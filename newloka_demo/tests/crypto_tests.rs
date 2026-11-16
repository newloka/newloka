use newloka_core::crypto::DeviceMasterKey;
use newloka_core::storage::StorageEngine;
use newloka_demo::{seed_demo_data, DEMO_MASTER_KEY_BYTES};

#[tokio::test]
async fn test_demo_seeding_succeeds_with_demo_master_key() {
    let dmk = DeviceMasterKey {
        key: DEMO_MASTER_KEY_BYTES,
    };
    let storage = StorageEngine::open(
        "sqlite::memory:?cache=shared",
        "test-demo-node".to_string(),
        dmk,
    )
    .await
    .expect("Failed to open in-memory storage");

    let result = seed_demo_data(&storage, "test-demo-node").await;
    assert!(result.is_ok(), "Demo seeding should succeed with correct Demo Master Key: {:?}", result.err());

    // Verify records can be retrieved and decrypted
    let config = storage.get_json("instance-config").await.expect("Failed to get config");
    assert!(config.is_some(), "instance-config must exist after demo seeding");

    let prescriber = storage.get_json("prescriber-profile").await.expect("Failed to get prescriber");
    assert!(prescriber.is_some(), "prescriber-profile must exist after demo seeding");

    let census = storage.get_json("inpatient-census").await.expect("Failed to get census");
    assert!(census.is_some(), "inpatient-census must exist after demo seeding");

    let patients = storage.search_json("Patient", None).await.expect("Failed to search patients");
    assert_eq!(patients.len(), 10, "Should have 10 demo patients seeded");
}

#[tokio::test]
async fn test_demo_seeding_rejected_with_non_demo_master_key() {
    // A customer's genuine or random key
    let customer_dmk = DeviceMasterKey {
        key: [0x42u8; 32],
    };
    let storage = StorageEngine::open(
        "sqlite::memory:?cache=shared",
        "customer-hospital-node".to_string(),
        customer_dmk,
    )
    .await
    .expect("Failed to open in-memory storage");

    let result = seed_demo_data(&storage, "customer-hospital-node").await;
    assert!(result.is_err(), "Seeding demo fixtures with customer key MUST fail cryptographically");

    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Cryptographic key mismatch"),
        "Error message must specify cryptographic key mismatch, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_demo_fixture_files_are_strictly_encrypted() {
    use newloka_demo::DemoFixtures;

    let manifest_file = DemoFixtures::get("manifest.json")
        .expect("manifest.json must be present in embedded fixtures");
    let manifest: Vec<serde_json::Value> =
        serde_json::from_slice(&manifest_file.data).expect("Valid manifest JSON");

    assert!(!manifest.is_empty(), "Manifest must not be empty");

    for item in &manifest {
        let filename = item["file"].as_str().unwrap();
        let path = format!("records/{}", filename);
        let record_bytes = DemoFixtures::get(&path)
            .unwrap_or_else(|| panic!("Record file {} must exist in fixtures", path));

        let content = std::str::from_utf8(&record_bytes.data).expect("Valid utf-8 JSON");

        // The serialized EncryptedRecord MUST NOT contain plaintext clinical identifiers
        assert!(
            !content.contains("Ramesh Patel"),
            "Encrypted fixture {} contains unencrypted patient name!",
            filename
        );
        assert!(
            !content.contains("Alex Mercer"),
            "Encrypted fixture {} contains unencrypted doctor name!",
            filename
        );
        assert!(
            !content.contains("Metformin"),
            "Encrypted fixture {} contains unencrypted medication name!",
            filename
        );
    }
}
