use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use futures::lock::Mutex;

use crate::popups::Popups;

pub fn routes(popups: Arc<Mutex<Popups>>) -> Router {
    Router::new()
        .route("/:id", get(create_order))
        .route("/:id", delete(serve_order))
        .with_state(popups)
}

async fn create_order(
    Path(_id): Path<u64>,
    State(_popups): State<Arc<Mutex<Popups>>>,
) -> impl IntoResponse {
    "Ok"
}

async fn serve_order(
    Path(id): Path<u64>,
    State(popups): State<Arc<Mutex<Popups>>>,
) -> impl IntoResponse {
    popups.lock().await.add_popup(id.to_string());
    "Ok"
}
