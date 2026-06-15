#[cfg(feature = "demo")]
pub async fn seed_demo_data(
    storage: &newloka_core::storage::StorageEngine,
    node_id: &str,
) -> anyhow::Result<()> {
    use chrono::Utc;

    let cities = [
        "Mumbai",
        "Delhi",
        "Bangalore",
        "Chennai",
        "Kolkata",
        "Hyderabad",
        "Pune",
        "Ahmedabad",
        "Jaipur",
        "Lucknow",
    ];

    let conditions = [
        ("Type 2 Diabetes Mellitus", "E11.9"),
        ("Essential Hypertension", "I10"),
        ("Asthma", "J45.9"),
        ("Acute Bronchitis", "J20.9"),
        ("Coronary Artery Disease", "I25.10"),
        ("Chronic Kidney Disease", "N18.9"),
        ("Osteoarthritis", "M19.90"),
        ("Major Depressive Disorder", "F32.9"),
        ("Migraine", "G43.909"),
        ("Appendicitis", "K35.80"),
    ];

    for i in 0..15 {
        let city = cities[i % cities.len()];
        let patient_id = uuid::Uuid::new_v4().to_string();
        let gender = if i % 2 == 0 { "male" } else { "female" };
        let given = if i % 2 == 0 { "Rajesh" } else { "Priya" };
        let family = if i % 3 == 0 { "Sharma" } else { "Patel" };
        let birth_year = 1950 + (i * 3) % 60;
        let birth_date = format!("{}-{:02}-{:02}", birth_year, (i % 12) + 1, (i % 28) + 1);

        let patient = serde_json::json!({
            "resourceType": "Patient",
            "id": patient_id,
            "meta": {
                "versionId": "1",
                "lastUpdated": Utc::now().to_rfc3339(),
                "sourceNodeId": node_id
            },
            "identifier": [{ "system": "http://newloka.org/mrn", "value": format!("MRN{:06}", 100000 + i) }],
            "active": true,
            "name": [{ "family": family, "given": [given] }],
            "gender": gender,
            "birthDate": birth_date,
            "telecom": [{ "system": "phone", "value": format!("+91-{}{}", 7000000000 + i as u64, i) }],
            "address": [{ "city": city, "country": "India" }]
        });
        storage
            .store_json("Patient", &patient_id, &patient, None, None)
            .await?;

        let enc_id = uuid::Uuid::new_v4().to_string();
        let encounter = serde_json::json!({
            "resourceType": "Encounter",
            "id": enc_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "finished",
            "class": { "system": "http://hl7.org/fhir/v3/ActCode", "code": "AMB", "display": "Ambulatory" },
            "subject": { "reference": format!("Patient/{}", patient_id) },
            "period": {
                "start": Utc::now().checked_sub_signed(chrono::Duration::days((i % 30) as i64)).unwrap_or(Utc::now()).to_rfc3339(),
                "end": Utc::now().to_rfc3339()
            },
            "location": [{ "location": { "display": city } }]
        });
        storage
            .store_json(
                "Encounter",
                &enc_id,
                &encounter,
                Some(patient_id.clone()),
                None,
            )
            .await?;

        let bp_id = uuid::Uuid::new_v4().to_string();
        let systolic = 110 + (i * 7) % 40;
        let diastolic = 70 + (i * 5) % 30;
        let bp = serde_json::json!({
            "resourceType": "Observation",
            "id": bp_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "final",
            "category": [{ "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/observation-category", "code": "vital-signs" }] }],
            "code": { "text": "Blood Pressure", "coding": [{ "system": "http://loinc.org", "code": "85354-9", "display": "Blood pressure panel" }] },
            "subject": { "reference": format!("Patient/{}", patient_id) },
            "effectiveDateTime": Utc::now().to_rfc3339(),
            "component": [
                { "code": { "text": "Systolic" }, "valueQuantity": { "value": systolic, "unit": "mmHg" } },
                { "code": { "text": "Diastolic" }, "valueQuantity": { "value": diastolic, "unit": "mmHg" } }
            ]
        });
        storage
            .store_json("Observation", &bp_id, &bp, Some(patient_id.clone()), None)
            .await?;

        let temp_id = uuid::Uuid::new_v4().to_string();
        let temp_val: f64 = 36.5 + ((i % 10) as f64) * 0.2;
        let temp = serde_json::json!({
            "resourceType": "Observation",
            "id": temp_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "final",
            "category": [{ "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/observation-category", "code": "vital-signs" }] }],
            "code": { "text": "Body Temperature", "coding": [{ "system": "http://loinc.org", "code": "8310-5", "display": "Body temperature" }] },
            "subject": { "reference": format!("Patient/{}", patient_id) },
            "effectiveDateTime": Utc::now().to_rfc3339(),
            "valueQuantity": { "value": temp_val, "unit": "degC" }
        });
        storage
            .store_json(
                "Observation",
                &temp_id,
                &temp,
                Some(patient_id.clone()),
                None,
            )
            .await?;

        if i % 3 == 0 {
            let lab_id = uuid::Uuid::new_v4().to_string();
            let hba1c: f64 = 5.5 + ((i % 15) as f64) * 0.3;
            let lab = serde_json::json!({
                "resourceType": "Observation",
                "id": lab_id,
                "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
                "status": "final",
                "category": [{ "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/observation-category", "code": "laboratory" }] }],
                "code": { "text": "HbA1c", "coding": [{ "system": "http://loinc.org", "code": "4548-4", "display": "Hemoglobin A1c" }] },
                "subject": { "reference": format!("Patient/{}", patient_id) },
                "effectiveDateTime": Utc::now().to_rfc3339(),
                "valueQuantity": { "value": hba1c, "unit": "%" },
                "interpretation": [{ "coding": [{ "system": "http://hl7.org/fhir/v3/ObservationInterpretation", "code": if hba1c < 5.7 { "N" } else if hba1c < 6.5 { "W" } else { "H" } }] }]
            });
            storage
                .store_json("Observation", &lab_id, &lab, Some(patient_id.clone()), None)
                .await?;
        }

        if i % 4 == 0 {
            let (text, code) = conditions[i % conditions.len()];
            let cond_id = uuid::Uuid::new_v4().to_string();
            let condition = serde_json::json!({
                "resourceType": "Condition",
                "id": cond_id,
                "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
                "verificationStatus": { "coding": [{ "code": "confirmed" }] },
                "code": { "text": text, "coding": [{ "system": "http://hl7.org/fhir/sid/icd-10", "code": code }] },
                "subject": { "reference": format!("Patient/{}", patient_id) },
                "onsetDateTime": format!("{}-06-15", 2015 + i % 8),
                "severity": { "text": if i % 2 == 0 { "mild" } else { "moderate" } }
            });
            storage
                .store_json(
                    "Condition",
                    &cond_id,
                    &condition,
                    Some(patient_id.clone()),
                    None,
                )
                .await?;
        }
    }

    let signer = newloka_core::crypto::AuditSigner::generate();
    let mut audit = newloka_core::audit::AuditEngine::new(node_id.to_string(), signer);
    for i in 0..10 {
        let entry = audit.log(
            newloka_core::audit::AuditEventType::Access,
            format!("user-{}", i),
            None,
            None,
            Some("Patient".to_string()),
            if i % 3 == 0 {
                "read".to_string()
            } else {
                "create".to_string()
            },
            newloka_core::audit::AuditOutcome::Success,
            Some(format!("Seeded audit entry {}", i)),
        )?;
        storage.store_audit(&entry).await?;
    }
    tracing::info!("Seeded demo dataset");
    Ok(())
}

#[cfg(not(feature = "demo"))]
pub async fn seed_demo_data(
    _storage: &newloka_core::storage::StorageEngine,
    _node_id: &str,
) -> anyhow::Result<()> {
    Ok(())
}
