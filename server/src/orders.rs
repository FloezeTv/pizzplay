use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
    Router,
};

use crate::events::{self, EventType};

pub fn routes(add_event: Arc<events::EventAddFunction>) -> Router {
    Router::new()
        .route("/:id", get(create_order))
        .route("/:id", delete(serve_order))
        .with_state(add_event)
}

async fn create_order(
    Path(_id): Path<u64>,
    State(_add_event): State<Arc<events::EventAddFunction>>,
) -> impl IntoResponse {
    "Ok"
}

async fn serve_order(
    Path(id): Path<u64>,
    State(add_event): State<Arc<events::EventAddFunction>>,
) -> impl IntoResponse {
    let _ = add_event(EventType::PopupShow, format!("{id}"));
    "Ok"
}
