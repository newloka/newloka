//! RxPad Integration & Prescription Routes

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/api/rxpad/sync", post(crate::rxpad::sync_prescription))
        .route(
            "/api/rxpad/prescriptions",
            get(crate::rxpad::list_prescriptions),
        )
        .route(
            "/api/rxpad/prescriptions/:id",
            get(crate::rxpad::get_prescription).delete(crate::rxpad::delete_prescription),
        )
        .route(
            "/api/rxpad/prescriptions/:id/dispense",
            post(crate::rxpad::dispense_prescription),
        )
        .route("/api/rxpad/next-serial", get(crate::rxpad::get_next_serial))
        .route("/api/rxpad/patients", get(crate::rxpad::search_patients))
        .route(
            "/api/rxpad/formulary",
            get(crate::rxpad::get_formulary).post(crate::rxpad::update_formulary),
        )
        .route("/api/rxpad/events", get(crate::rxpad::rxpad_events_handler))
        .route(
            "/api/rxpad/template",
            get(crate::rxpad::get_template).post(crate::rxpad::save_template),
        )
        .route("/rxpad", get(crate::rxpad::serve_rxpad))
}
