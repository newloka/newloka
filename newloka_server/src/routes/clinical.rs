//! FHIR R4 Clinical Resources CRUD Routes
//!
//! Exposes strongly-typed FHIR endpoints backed by AES-256-GCM encrypted storage.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::routes::resource_helpers::{
    create_resource, delete_resource, get_resource, search_resource, update_resource, SearchParams,
};
use crate::state::AppState;

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        /* Patient */
        .route("/Patient", get(search_patients).post(create_patient))
        .route(
            "/Patient/:id",
            get(get_patient).put(update_patient).delete(delete_patient),
        )
        /* Encounter */
        .route("/Encounter", get(search_encounters).post(create_encounter))
        .route(
            "/Encounter/:id",
            get(get_encounter).put(update_encounter).delete(delete_encounter),
        )
        /* Observation */
        .route("/Observation", get(search_observations).post(create_observation))
        .route(
            "/Observation/:id",
            get(get_observation).put(update_observation).delete(delete_observation),
        )
        /* Condition */
        .route("/Condition", get(search_conditions).post(create_condition))
        .route(
            "/Condition/:id",
            get(get_condition).put(update_condition).delete(delete_condition),
        )
        /* MedicationRequest */
        .route(
            "/MedicationRequest",
            get(search_medication_requests).post(create_medication_request),
        )
        .route(
            "/MedicationRequest/:id",
            get(get_medication_request)
                .put(update_medication_request)
                .delete(delete_medication_request),
        )
        /* MedicationDispense */
        .route(
            "/MedicationDispense",
            get(search_medication_dispenses).post(create_medication_dispense),
        )
        .route(
            "/MedicationDispense/:id",
            get(get_medication_dispense)
                .put(update_medication_dispense)
                .delete(delete_medication_dispense),
        )
        /* Procedure */
        .route("/Procedure", get(search_procedures).post(create_procedure))
        .route(
            "/Procedure/:id",
            get(get_procedure).put(update_procedure).delete(delete_procedure),
        )
        /* AllergyIntolerance */
        .route(
            "/AllergyIntolerance",
            get(search_allergy_intolerances).post(create_allergy_intolerance),
        )
        .route(
            "/AllergyIntolerance/:id",
            get(get_allergy_intolerance)
                .put(update_allergy_intolerance)
                .delete(delete_allergy_intolerance),
        )
        /* Flag */
        .route("/Flag", get(search_flags).post(create_flag))
        .route(
            "/Flag/:id",
            get(get_flag).put(update_flag).delete(delete_flag),
        )
        /* DetectedIssue */
        .route(
            "/DetectedIssue",
            get(search_detected_issues).post(create_detected_issue),
        )
        .route(
            "/DetectedIssue/:id",
            get(get_detected_issue)
                .put(update_detected_issue)
                .delete(delete_detected_issue),
        )
        /* Composition */
        .route(
            "/Composition",
            get(search_compositions).post(create_composition),
        )
        .route(
            "/Composition/:id",
            get(get_composition)
                .put(update_composition)
                .delete(delete_composition),
        )
        /* CarePlan */
        .route("/CarePlan", get(search_care_plans).post(create_care_plan))
        .route(
            "/CarePlan/:id",
            get(get_care_plan).put(update_care_plan).delete(delete_care_plan),
        )
        /* FamilyMemberHistory */
        .route(
            "/FamilyMemberHistory",
            get(search_family_member_histories).post(create_family_member_history),
        )
        .route(
            "/FamilyMemberHistory/:id",
            get(get_family_member_history)
                .put(update_family_member_history)
                .delete(delete_family_member_history),
        )
        /* Immunization */
        .route(
            "/Immunization",
            get(search_immunizations).post(create_immunization),
        )
        .route(
            "/Immunization/:id",
            get(get_immunization)
                .put(update_immunization)
                .delete(delete_immunization),
        )
        /* DocumentReference */
        .route(
            "/DocumentReference",
            get(search_document_references).post(create_document_reference),
        )
        .route(
            "/DocumentReference/:id",
            get(get_document_reference)
                .put(update_document_reference)
                .delete(delete_document_reference),
        )
        /* CPOE: ServiceRequest */
        .route(
            "/ServiceRequest",
            get(search_service_requests).post(create_service_request),
        )
        .route(
            "/ServiceRequest/:id",
            get(get_service_request)
                .put(update_service_request)
                .delete(delete_service_request),
        )
        /* CPOE: ImagingStudy */
        .route(
            "/ImagingStudy",
            get(search_imaging_studies).post(create_imaging_study),
        )
        .route(
            "/ImagingStudy/:id",
            get(get_imaging_study)
                .put(update_imaging_study)
                .delete(delete_imaging_study),
        )
        /* CPOE: Specimen */
        .route("/Specimen", get(search_specimens).post(create_specimen))
        .route(
            "/Specimen/:id",
            get(get_specimen).put(update_specimen).delete(delete_specimen),
        )
        /* MedicationAdministration */
        .route(
            "/MedicationAdministration",
            get(search_medication_administrations).post(create_medication_administration),
        )
        .route(
            "/MedicationAdministration/:id",
            get(get_medication_administration)
                .put(update_medication_administration)
                .delete(delete_medication_administration),
        )
        /* MedicationStatement */
        .route(
            "/MedicationStatement",
            get(search_medication_statements).post(create_medication_statement),
        )
        .route(
            "/MedicationStatement/:id",
            get(get_medication_statement)
                .put(update_medication_statement)
                .delete(delete_medication_statement),
        )
        /* Whiteboard */
        .route("/Whiteboard", get(search_whiteboards).post(create_whiteboard))
        .route(
            "/Whiteboard/:id",
            get(get_whiteboard)
                .put(update_whiteboard)
                .delete(delete_whiteboard),
        )
        /* Provenance */
        .route("/Provenance", get(search_provenances).post(create_provenance))
        .route(
            "/Provenance/:id",
            get(get_provenance)
                .put(update_provenance)
                .delete(delete_provenance),
        )
}

macro_rules! resource_handlers {
    ($search_fn:ident, $create_fn:ident, $get_fn:ident, $update_fn:ident, $delete_fn:ident, $res_type:literal) => {
        pub async fn $search_fn(
            s: State<Arc<RwLock<AppState>>>,
            q: Query<SearchParams>,
        ) -> (StatusCode, Json<serde_json::Value>) {
            search_resource(s, q, $res_type).await
        }

        pub async fn $create_fn(
            s: State<Arc<RwLock<AppState>>>,
            b: Json<serde_json::Value>,
        ) -> (StatusCode, Json<serde_json::Value>) {
            create_resource(s, b, $res_type).await
        }

        pub async fn $get_fn(
            s: State<Arc<RwLock<AppState>>>,
            p: Path<String>,
        ) -> (StatusCode, Json<serde_json::Value>) {
            get_resource(s, p, $res_type).await
        }

        pub async fn $update_fn(
            s: State<Arc<RwLock<AppState>>>,
            p: Path<String>,
            b: Json<serde_json::Value>,
        ) -> (StatusCode, Json<serde_json::Value>) {
            update_resource(s, p, b, $res_type).await
        }

        pub async fn $delete_fn(s: State<Arc<RwLock<AppState>>>, p: Path<String>) -> StatusCode {
            delete_resource(s, p, $res_type).await
        }
    };
}

resource_handlers!(
    search_patients,
    create_patient,
    get_patient,
    update_patient,
    delete_patient,
    "Patient"
);
resource_handlers!(
    search_encounters,
    create_encounter,
    get_encounter,
    update_encounter,
    delete_encounter,
    "Encounter"
);
resource_handlers!(
    search_observations,
    create_observation,
    get_observation,
    update_observation,
    delete_observation,
    "Observation"
);
resource_handlers!(
    search_conditions,
    create_condition,
    get_condition,
    update_condition,
    delete_condition,
    "Condition"
);
resource_handlers!(
    search_medication_requests,
    create_medication_request,
    get_medication_request,
    update_medication_request,
    delete_medication_request,
    "MedicationRequest"
);
resource_handlers!(
    search_medication_dispenses,
    create_medication_dispense,
    get_medication_dispense,
    update_medication_dispense,
    delete_medication_dispense,
    "MedicationDispense"
);
resource_handlers!(
    search_procedures,
    create_procedure,
    get_procedure,
    update_procedure,
    delete_procedure,
    "Procedure"
);
resource_handlers!(
    search_allergy_intolerances,
    create_allergy_intolerance,
    get_allergy_intolerance,
    update_allergy_intolerance,
    delete_allergy_intolerance,
    "AllergyIntolerance"
);
resource_handlers!(
    search_flags,
    create_flag,
    get_flag,
    update_flag,
    delete_flag,
    "Flag"
);
resource_handlers!(
    search_detected_issues,
    create_detected_issue,
    get_detected_issue,
    update_detected_issue,
    delete_detected_issue,
    "DetectedIssue"
);
resource_handlers!(
    search_compositions,
    create_composition,
    get_composition,
    update_composition,
    delete_composition,
    "Composition"
);
resource_handlers!(
    search_care_plans,
    create_care_plan,
    get_care_plan,
    update_care_plan,
    delete_care_plan,
    "CarePlan"
);
resource_handlers!(
    search_family_member_histories,
    create_family_member_history,
    get_family_member_history,
    update_family_member_history,
    delete_family_member_history,
    "FamilyMemberHistory"
);
resource_handlers!(
    search_immunizations,
    create_immunization,
    get_immunization,
    update_immunization,
    delete_immunization,
    "Immunization"
);
resource_handlers!(
    search_document_references,
    create_document_reference,
    get_document_reference,
    update_document_reference,
    delete_document_reference,
    "DocumentReference"
);
resource_handlers!(
    search_service_requests,
    create_service_request,
    get_service_request,
    update_service_request,
    delete_service_request,
    "ServiceRequest"
);
resource_handlers!(
    search_imaging_studies,
    create_imaging_study,
    get_imaging_study,
    update_imaging_study,
    delete_imaging_study,
    "ImagingStudy"
);
resource_handlers!(
    search_specimens,
    create_specimen,
    get_specimen,
    update_specimen,
    delete_specimen,
    "Specimen"
);
resource_handlers!(
    search_medication_administrations,
    create_medication_administration,
    get_medication_administration,
    update_medication_administration,
    delete_medication_administration,
    "MedicationAdministration"
);
resource_handlers!(
    search_medication_statements,
    create_medication_statement,
    get_medication_statement,
    update_medication_statement,
    delete_medication_statement,
    "MedicationStatement"
);
resource_handlers!(
    search_whiteboards,
    create_whiteboard,
    get_whiteboard,
    update_whiteboard,
    delete_whiteboard,
    "Whiteboard"
);
resource_handlers!(
    search_provenances,
    create_provenance,
    get_provenance,
    update_provenance,
    delete_provenance,
    "Provenance"
);
