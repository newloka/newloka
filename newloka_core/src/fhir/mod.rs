//! FHIR R4 Resource Models
//!
//! Canonical internal data model. Every clinical record maps to a FHIR R4 resource.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base fields present on every FHIR resource in New Loka.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub version_id: String,
    pub last_updated: DateTime<Utc>,
    pub source_node_id: String,
    pub created_by: String,
    pub modified_by: String,
    pub vector_clock: Vec<(u32, u64)>,
    pub provenance: Vec<ProvenanceEntry>,
}

impl Meta {
    pub fn new(node_id: String, user_id: String) -> Self {
        Self {
            version_id: Uuid::new_v4().to_string(),
            last_updated: Utc::now(),
            source_node_id: node_id,
            created_by: user_id.clone(),
            modified_by: user_id,
            vector_clock: vec![],
            provenance: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceEntry {
    pub action: String,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub node_id: String,
    pub reason: Option<String>,
}

/// FHIR Resource wrapper. All clinical records are stored as one of these variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "resourceType")]
pub enum FhirResource {
    Patient(Patient),
    Encounter(Encounter),
    Observation(Observation),
    Condition(Condition),
    MedicationRequest(MedicationRequest),
    Procedure(Procedure),
    DiagnosticReport(DiagnosticReport),
    ServiceRequest(ServiceRequest),
    ImagingStudy(ImagingStudy),
    Specimen(Specimen),
    Composition(Composition),
    DetectedIssue(DetectedIssue),
    AllergyIntolerance(AllergyIntolerance),
    Flag(Flag),
    CarePlan(CarePlan),
    FamilyMemberHistory(FamilyMemberHistory),
    Immunization(Immunization),
    DocumentReference(DocumentReference),
    AuditEvent(AuditEvent),
    Provenance(Provenance),
    Bundle(Bundle),
}

impl FhirResource {
    pub fn id(&self) -> &str {
        match self {
            FhirResource::Patient(r) => &r.id,
            FhirResource::Encounter(r) => &r.id,
            FhirResource::Observation(r) => &r.id,
            FhirResource::Condition(r) => &r.id,
            FhirResource::MedicationRequest(r) => &r.id,
            FhirResource::Procedure(r) => &r.id,
            FhirResource::DiagnosticReport(r) => &r.id,
            FhirResource::ServiceRequest(r) => &r.id,
            FhirResource::ImagingStudy(r) => &r.id,
            FhirResource::Specimen(r) => &r.id,
            FhirResource::Composition(r) => &r.id,
            FhirResource::DetectedIssue(r) => &r.id,
            FhirResource::AllergyIntolerance(r) => &r.id,
            FhirResource::Flag(r) => &r.id,
            FhirResource::CarePlan(r) => &r.id,
            FhirResource::FamilyMemberHistory(r) => &r.id,
            FhirResource::Immunization(r) => &r.id,
            FhirResource::DocumentReference(r) => &r.id,
            FhirResource::AuditEvent(r) => &r.id,
            FhirResource::Provenance(r) => &r.id,
            FhirResource::Bundle(r) => &r.id,
        }
    }

    pub fn meta(&self) -> &Meta {
        match self {
            FhirResource::Patient(r) => &r.meta,
            FhirResource::Encounter(r) => &r.meta,
            FhirResource::Observation(r) => &r.meta,
            FhirResource::Condition(r) => &r.meta,
            FhirResource::MedicationRequest(r) => &r.meta,
            FhirResource::Procedure(r) => &r.meta,
            FhirResource::DiagnosticReport(r) => &r.meta,
            FhirResource::ServiceRequest(r) => &r.meta,
            FhirResource::ImagingStudy(r) => &r.meta,
            FhirResource::Specimen(r) => &r.meta,
            FhirResource::Composition(r) => &r.meta,
            FhirResource::DetectedIssue(r) => &r.meta,
            FhirResource::AllergyIntolerance(r) => &r.meta,
            FhirResource::Flag(r) => &r.meta,
            FhirResource::CarePlan(r) => &r.meta,
            FhirResource::FamilyMemberHistory(r) => &r.meta,
            FhirResource::Immunization(r) => &r.meta,
            FhirResource::DocumentReference(r) => &r.meta,
            FhirResource::AuditEvent(r) => &r.meta,
            FhirResource::Provenance(r) => &r.meta,
            FhirResource::Bundle(r) => &r.meta,
        }
    }

    pub fn resource_type(&self) -> &str {
        match self {
            FhirResource::Patient(_) => "Patient",
            FhirResource::Encounter(_) => "Encounter",
            FhirResource::Observation(_) => "Observation",
            FhirResource::Condition(_) => "Condition",
            FhirResource::MedicationRequest(_) => "MedicationRequest",
            FhirResource::Procedure(_) => "Procedure",
            FhirResource::DiagnosticReport(_) => "DiagnosticReport",
            FhirResource::ServiceRequest(_) => "ServiceRequest",
            FhirResource::ImagingStudy(_) => "ImagingStudy",
            FhirResource::Specimen(_) => "Specimen",
            FhirResource::Composition(_) => "Composition",
            FhirResource::DetectedIssue(_) => "DetectedIssue",
            FhirResource::AllergyIntolerance(_) => "AllergyIntolerance",
            FhirResource::Flag(_) => "Flag",
            FhirResource::CarePlan(_) => "CarePlan",
            FhirResource::FamilyMemberHistory(_) => "FamilyMemberHistory",
            FhirResource::Immunization(_) => "Immunization",
            FhirResource::DocumentReference(_) => "DocumentReference",
            FhirResource::AuditEvent(_) => "AuditEvent",
            FhirResource::Provenance(_) => "Provenance",
            FhirResource::Bundle(_) => "Bundle",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct HumanName {
    pub use_field: Option<String>,
    pub family: String,
    pub given: Vec<String>,
    pub prefix: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Identifier {
    pub system: String,
    pub value: String,
    pub use_field: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContactPoint {
    pub system: String,
    pub value: String,
    pub use_field: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub use_field: Option<String>,
    pub line: Vec<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CodeableConcept {
    pub text: Option<String>,
    pub coding: Vec<Coding>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Coding {
    pub system: String,
    pub code: String,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Reference {
    pub reference: String,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
    pub system: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Period {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Patient {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub active: bool,
    pub name: Vec<HumanName>,
    pub telecom: Vec<ContactPoint>,
    pub gender: String,
    pub birth_date: String,
    pub address: Vec<Address>,
    pub marital_status: Option<CodeableConcept>,
    pub general_practitioner: Vec<Reference>,
    pub managing_organization: Option<Reference>,
    pub deceased_boolean: Option<bool>,
    pub deceased_date_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Encounter {
    pub id: String,
    pub meta: Meta,
    pub status: String,
    pub class: Coding,
    pub type_: Vec<CodeableConcept>,
    pub subject: Reference,
    pub participant: Vec<EncounterParticipant>,
    pub period: Period,
    pub location: Vec<EncounterLocation>,
    pub reason_code: Vec<CodeableConcept>,
    pub diagnosis: Vec<EncounterDiagnosis>,
    pub service_provider: Option<Reference>,
    pub part_of: Option<Reference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EncounterParticipant {
    pub type_: Vec<CodeableConcept>,
    pub period: Option<Period>,
    pub individual: Reference,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EncounterLocation {
    pub location: Reference,
    pub status: Option<String>,
    pub period: Option<Period>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EncounterDiagnosis {
    pub condition: Reference,
    pub use_: Option<CodeableConcept>,
    pub rank: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    pub id: String,
    pub meta: Meta,
    pub status: String,
    pub category: Vec<CodeableConcept>,
    pub code: CodeableConcept,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub effective_date_time: Option<DateTime<Utc>>,
    pub issued: Option<DateTime<Utc>>,
    pub performer: Vec<Reference>,
    pub value_quantity: Option<Quantity>,
    pub value_string: Option<String>,
    pub value_codeable_concept: Option<CodeableConcept>,
    pub interpretation: Vec<CodeableConcept>,
    pub note: Vec<Annotation>,
    pub reference_range: Vec<ObservationReferenceRange>,
    pub component: Vec<ObservationComponent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Annotation {
    pub author_string: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ObservationReferenceRange {
    pub low: Option<Quantity>,
    pub high: Option<Quantity>,
    pub type_: Option<CodeableConcept>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ObservationComponent {
    pub code: CodeableConcept,
    pub value_quantity: Option<Quantity>,
    pub value_string: Option<String>,
    pub interpretation: Vec<CodeableConcept>,
    pub reference_range: Vec<ObservationReferenceRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub clinical_status: CodeableConcept,
    pub verification_status: Option<CodeableConcept>,
    pub category: Vec<CodeableConcept>,
    pub severity: Option<CodeableConcept>,
    pub code: CodeableConcept,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub onset_date_time: Option<DateTime<Utc>>,
    pub abatement_date_time: Option<DateTime<Utc>>,
    pub recorded_date: Option<DateTime<Utc>>,
    pub recorder: Option<Reference>,
    pub asserter: Option<Reference>,
    pub note: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MedicationRequest {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub intent: String,
    pub medication_codeable_concept: Option<CodeableConcept>,
    pub medication_reference: Option<Reference>,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub authored_on: DateTime<Utc>,
    pub requester: Reference,
    pub dosage_instruction: Vec<Dosage>,
    pub dispense_request: Option<MedicationRequestDispense>,
    pub substitution: Option<MedicationRequestSubstitution>,
    pub prior_prescription: Option<Reference>,
    pub detected_issue: Vec<Reference>,
    pub event_history: Vec<Reference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Dosage {
    pub sequence: Option<i32>,
    pub text: Option<String>,
    pub additional_instruction: Vec<CodeableConcept>,
    pub timing: Option<Timing>,
    pub route: Option<CodeableConcept>,
    pub method: Option<CodeableConcept>,
    pub dose_quantity: Option<Quantity>,
    pub as_needed_boolean: Option<bool>,
    pub max_dose_per_period: Option<Ratio>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Timing {
    pub repeat: Option<TimingRepeat>,
    pub code: Option<CodeableConcept>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TimingRepeat {
    pub frequency: Option<i32>,
    pub period: Option<f64>,
    pub period_unit: Option<String>,
    pub when: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Ratio {
    pub numerator: Option<Quantity>,
    pub denominator: Option<Quantity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MedicationRequestDispense {
    pub number_of_repeats_allowed: Option<i32>,
    pub quantity: Option<Quantity>,
    pub expected_supply_duration: Option<Quantity>,
    pub performer: Option<Reference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MedicationRequestSubstitution {
    pub allowed_boolean: bool,
    pub reason: Option<CodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Procedure {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub category: Option<CodeableConcept>,
    pub code: CodeableConcept,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub performed_date_time: Option<DateTime<Utc>>,
    pub performed_period: Option<Period>,
    pub recorder: Option<Reference>,
    pub performer: Vec<ProcedurePerformer>,
    pub location: Option<Reference>,
    pub reason_code: Vec<CodeableConcept>,
    pub body_site: Vec<CodeableConcept>,
    pub outcome: Option<CodeableConcept>,
    pub report: Vec<Reference>,
    pub complication: Vec<CodeableConcept>,
    pub follow_up: Vec<CodeableConcept>,
    pub note: Vec<Annotation>,
    pub used_reference: Vec<Reference>,
    pub used_code: Vec<CodeableConcept>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProcedurePerformer {
    pub function_: Option<CodeableConcept>,
    pub actor: Reference,
    pub on_behalf_of: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticReport {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub category: Vec<CodeableConcept>,
    pub code: CodeableConcept,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub effective_date_time: Option<DateTime<Utc>>,
    pub effective_period: Option<Period>,
    pub issued: Option<DateTime<Utc>>,
    pub performer: Vec<Reference>,
    pub results_interpreter: Vec<Reference>,
    pub specimen: Vec<Reference>,
    pub result: Vec<Reference>,
    pub imaging_study: Vec<Reference>,
    pub media: Vec<DiagnosticReportMedia>,
    pub conclusion: Option<String>,
    pub conclusion_code: Vec<CodeableConcept>,
    pub presented_form: Vec<Attachment>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticReportMedia {
    pub comment: Option<String>,
    pub link: Reference,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    pub content_type: Option<String>,
    pub language: Option<String>,
    pub data: Option<String>,
    pub url: Option<String>,
    pub size: Option<i64>,
    pub title: Option<String>,
    pub creation: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Composition {
    pub id: String,
    pub meta: Meta,
    pub identifier: Option<Identifier>,
    pub status: String,
    pub type_: CodeableConcept,
    pub category: Vec<CodeableConcept>,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub date: DateTime<Utc>,
    pub author: Vec<Reference>,
    pub title: String,
    pub confidentiality: Option<String>,
    pub section: Vec<CompositionSection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CompositionSection {
    pub title: Option<String>,
    pub code: Option<CodeableConcept>,
    pub author: Vec<Reference>,
    pub text: Option<String>,
    pub mode: Option<String>,
    pub ordered_by: Option<CodeableConcept>,
    pub entry: Vec<Reference>,
    pub empty_reason: Option<CodeableConcept>,
    pub section: Vec<CompositionSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectedIssue {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub code: Option<CodeableConcept>,
    pub severity: Option<String>,
    pub patient: Option<Reference>,
    pub identified_date_time: Option<DateTime<Utc>>,
    pub identified_period: Option<Period>,
    pub author: Option<Reference>,
    pub implicated: Vec<Reference>,
    pub evidence: Vec<DetectedIssueEvidence>,
    pub detail: Option<String>,
    pub reference: Option<String>,
    pub mitigation: Vec<DetectedIssueMitigation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DetectedIssueEvidence {
    pub code: Vec<CodeableConcept>,
    pub detail: Vec<Reference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DetectedIssueMitigation {
    pub action: CodeableConcept,
    pub date: Option<DateTime<Utc>>,
    pub author: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub id: String,
    pub meta: Meta,
    pub type_: Coding,
    pub subtype: Vec<Coding>,
    pub action: Option<String>,
    pub period: Option<Period>,
    pub recorded: DateTime<Utc>,
    pub outcome: String,
    pub outcome_desc: Option<String>,
    pub purpose_of_event: Vec<CodeableConcept>,
    pub agent: Vec<AuditEventAgent>,
    pub source: AuditEventSource,
    pub entity: Vec<AuditEventEntity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditEventAgent {
    pub type_: Option<CodeableConcept>,
    pub role: Vec<CodeableConcept>,
    pub who: Reference,
    pub alt_id: Option<String>,
    pub name: Option<String>,
    pub requestor: bool,
    pub location: Option<Reference>,
    pub policy: Vec<String>,
    pub media: Option<Coding>,
    pub network: Option<AuditEventAgentNetwork>,
    pub purpose_of_use: Vec<CodeableConcept>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditEventAgentNetwork {
    pub address: Option<String>,
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditEventSource {
    pub site: Option<String>,
    pub observer: Reference,
    pub type_: Vec<Coding>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditEventEntity {
    pub what: Option<Reference>,
    pub type_: Option<Coding>,
    pub role: Option<Coding>,
    pub lifecycle: Option<Coding>,
    pub security_label: Vec<Coding>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub detail: Vec<AuditEventEntityDetail>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditEventEntityDetail {
    pub type_: String,
    pub value_string: Option<String>,
    pub value_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequest {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub intent: String,
    pub category: Vec<CodeableConcept>,
    pub priority: Option<String>,
    pub code: Option<CodeableConcept>,
    pub order_detail: Vec<CodeableConcept>,
    pub subject: Option<Reference>,
    pub encounter: Option<Reference>,
    pub authored_on: Option<DateTime<Utc>>,
    pub requester: Option<Reference>,
    pub performer: Vec<Reference>,
    pub performer_type: Option<CodeableConcept>,
    pub location_code: Vec<CodeableConcept>,
    pub reason_code: Vec<CodeableConcept>,
    pub insurance: Vec<Reference>,
    pub supporting_info: Vec<Reference>,
    pub specimen: Vec<Reference>,
    pub body_site: Vec<CodeableConcept>,
    pub note: Vec<Annotation>,
    pub patient_instruction: Option<String>,
    pub relevant_history: Vec<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImagingStudy {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub modality: Vec<Coding>,
    pub subject: Reference,
    pub encounter: Option<Reference>,
    pub started: Option<DateTime<Utc>>,
    pub based_on: Vec<Reference>,
    pub referrer: Option<Reference>,
    pub interpreter: Vec<Reference>,
    pub endpoint: Vec<Reference>,
    pub number_of_series: Option<i64>,
    pub number_of_instances: Option<i64>,
    pub procedure_reference: Vec<Reference>,
    pub procedure_code: Vec<CodeableConcept>,
    pub location: Option<Reference>,
    pub reason_code: Vec<CodeableConcept>,
    pub note: Vec<Annotation>,
    pub series: Vec<ImagingStudySeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImagingStudySeries {
    pub uid: String,
    pub number: Option<i64>,
    pub modality: Coding,
    pub description: Option<String>,
    pub number_of_instances: Option<i64>,
    pub endpoint: Vec<Reference>,
    pub body_site: Option<CodeableConcept>,
    pub instance: Vec<ImagingStudyInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImagingStudyInstance {
    pub uid: String,
    pub sop_class: Coding,
    pub number: Option<i64>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Specimen {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub accession_identifier: Option<Identifier>,
    pub status: Option<String>,
    pub type_: Option<CodeableConcept>,
    pub subject: Option<Reference>,
    pub received_time: Option<DateTime<Utc>>,
    pub parent: Vec<Reference>,
    pub request: Vec<Reference>,
    pub collection: Option<SpecimenCollection>,
    pub processing: Vec<SpecimenProcessing>,
    pub container: Vec<SpecimenContainer>,
    pub note: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpecimenCollection {
    pub collector: Option<Reference>,
    pub collected_date_time: Option<DateTime<Utc>>,
    pub collected_period: Option<Period>,
    pub quantity: Option<Quantity>,
    pub method: Option<CodeableConcept>,
    pub body_site: Option<CodeableConcept>,
    pub fasting_status: Option<CodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpecimenProcessing {
    pub description: Option<String>,
    pub procedure: Option<CodeableConcept>,
    pub additive: Vec<Reference>,
    pub time_date_time: Option<DateTime<Utc>>,
    pub time_period: Option<Period>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpecimenContainer {
    pub identifier: Vec<Identifier>,
    pub description: Option<String>,
    pub type_: Option<CodeableConcept>,
    pub capacity: Option<Quantity>,
    pub specimen_quantity: Option<Quantity>,
    pub additive: Option<CodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provenance {
    pub id: String,
    pub meta: Meta,
    pub target: Vec<Reference>,
    pub occurred_period: Option<Period>,
    pub recorded: DateTime<Utc>,
    pub policy: Vec<String>,
    pub location: Option<Reference>,
    pub reason: Vec<CodeableConcept>,
    pub activity: Option<CodeableConcept>,
    pub agent: Vec<ProvenanceAgent>,
    pub entity: Vec<ProvenanceEntity>,
    pub signature: Vec<Signature>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceAgent {
    pub type_: Option<CodeableConcept>,
    pub role: Vec<CodeableConcept>,
    pub who: Reference,
    pub on_behalf_of: Option<Reference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceEntity {
    pub role: String,
    pub what: Reference,
    pub agent: Vec<ProvenanceAgent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Signature {
    pub type_: Vec<Coding>,
    pub when: DateTime<Utc>,
    pub who: Reference,
    pub on_behalf_of: Option<Reference>,
    pub sig_format: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bundle {
    pub id: String,
    pub meta: Meta,
    pub identifier: Option<Identifier>,
    pub type_: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub total: Option<u32>,
    pub link: Vec<BundleLink>,
    pub entry: Vec<BundleEntry>,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleLink {
    pub relation: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleEntry {
    pub link: Vec<BundleLink>,
    pub full_url: Option<String>,
    pub resource: Option<FhirResource>,
    pub search: Option<BundleEntrySearch>,
    pub request: Option<BundleEntryRequest>,
    pub response: Option<BundleEntryResponse>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleEntrySearch {
    pub mode: Option<String>,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleEntryRequest {
    pub method: String,
    pub url: String,
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<DateTime<Utc>>,
    pub if_match: Option<String>,
    pub if_none_exist: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BundleEntryResponse {
    pub status: String,
    pub location: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub outcome: Option<Box<FhirResource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllergyIntolerance {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub clinical_status: Option<CodeableConcept>,
    pub verification_status: Option<CodeableConcept>,
    pub type_: Option<CodeableConcept>,
    pub category: Vec<String>,
    pub criticality: Option<String>,
    pub code: Option<CodeableConcept>,
    pub patient: Option<Reference>,
    pub onset_date_time: Option<DateTime<Utc>>,
    pub recorded_date: Option<DateTime<Utc>>,
    pub recorder: Option<Reference>,
    pub asserter: Option<Reference>,
    pub last_occurrence: Option<DateTime<Utc>>,
    pub note: Vec<Annotation>,
    pub reaction: Vec<AllergyIntoleranceReaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllergyIntoleranceReaction {
    pub substance: Option<CodeableConcept>,
    pub manifestation: Vec<CodeableConcept>,
    pub description: Option<String>,
    pub onset: Option<DateTime<Utc>>,
    pub severity: Option<String>,
    pub exposure_route: Option<CodeableConcept>,
    pub note: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub category: Vec<CodeableConcept>,
    pub code: Option<CodeableConcept>,
    pub subject: Option<Reference>,
    pub period: Option<Period>,
    pub encounter: Option<Reference>,
    pub author: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CarePlan {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub intent: String,
    pub category: Vec<CodeableConcept>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub subject: Option<Reference>,
    pub encounter: Option<Reference>,
    pub period: Option<Period>,
    pub author: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FamilyMemberHistory {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub patient: Option<Reference>,
    pub date: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub relationship: Option<CodeableConcept>,
    pub sex: Option<CodeableConcept>,
    pub born_string: Option<String>,
    pub age_string: Option<String>,
    pub reason_code: Vec<CodeableConcept>,
    pub condition: Vec<FamilyMemberHistoryCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FamilyMemberHistoryCondition {
    pub code: Option<CodeableConcept>,
    pub outcome: Option<CodeableConcept>,
    pub onset_string: Option<String>,
    pub note: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Immunization {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub status_reason: Option<CodeableConcept>,
    pub vaccine_code: Option<CodeableConcept>,
    pub patient: Option<Reference>,
    pub encounter: Option<Reference>,
    pub occurrence_date_time: Option<DateTime<Utc>>,
    pub recorded: Option<DateTime<Utc>>,
    pub primary_source: Option<bool>,
    pub location: Option<Reference>,
    pub manufacturer: Option<Reference>,
    pub lot_number: Option<String>,
    pub note: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentReference {
    pub id: String,
    pub meta: Meta,
    pub identifier: Vec<Identifier>,
    pub status: String,
    pub doc_status: Option<CodeableConcept>,
    pub type_: Option<CodeableConcept>,
    pub category: Vec<CodeableConcept>,
    pub subject: Option<Reference>,
    pub date: Option<DateTime<Utc>>,
    pub author: Vec<Reference>,
    pub description: Option<String>,
    pub content: Vec<DocumentReferenceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentReferenceContent {
    pub attachment: Option<Attachment>,
    pub format: Option<Coding>,
}
