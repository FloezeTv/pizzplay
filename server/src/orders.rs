use std::{error::Error, fs, sync::Arc, thread::current, time::{SystemTime, Duration}};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use futures::lock::Mutex;

use serde::{Deserialize, Serialize};

use crate::popups::Popups;

#[derive(Serialize, Deserialize, Clone)]
struct Order {
    pub timestamp: u128,
    #[serde(rename = "type")]
    pub order_type: String,
    pub number: u64,
}

struct OrderState {
    popups: Arc<Mutex<Popups>>,
    current: Arc<Mutex<Vec<Order>>>,
    all: Arc<Mutex<Vec<Order>>>,
}

pub fn routes(popups: Arc<Mutex<Popups>>) -> Router {
    let (current, all) = load().unwrap_or_else(|_| (Vec::new(), Vec::new()));
    Router::new()
        .route("/:id", post(create_order))
        .route("/:id", delete(serve_order))
        .route("/", get(statistics))
        .with_state(Arc::new(OrderState {
            popups,
            current: Arc::new(Mutex::new(current)),
            all: Arc::new(Mutex::new(all)),
        }))
}

async fn create_order(
    Path(id): Path<u64>,
    State(state): State<Arc<OrderState>>,
    body: String,
) -> impl IntoResponse {
    let order = Order {
        timestamp: SystemTime::UNIX_EPOCH.elapsed().unwrap_or(Duration::default()).as_millis(),
        order_type: body,
        number: id,
    };
    {state.all.lock().await.push(order.clone());};
    let mut current_state = state.current.lock().await;
    current_state.push(order.clone());
    Json(current_state.clone())
}

async fn serve_order(
    Path(id): Path<u64>,
    State(state): State<Arc<OrderState>>,
) -> impl IntoResponse {
    {state.popups.lock().await.add_popup(id.to_string())};
    let mut current = state.current.lock().await;
    let all = state.all.lock().await;
    current.retain(|e| e.number != id);
    save(&current, &all);
    Json(current.clone())
}

async fn statistics(State(state): State<Arc<OrderState>>) -> impl IntoResponse {
    Json(state.all.lock().await.clone())
}

fn save(current: &Vec<Order>, all: &Vec<Order>) {
    let rc = fs::write(
        "./current.json",
        serde_json::to_string(current).unwrap_or_else(|_| "[]".to_owned()),
    );
    let ra = fs::write(
        "./all.json",
        serde_json::to_string(all).unwrap_or_else(|_| "[]".to_owned()),
    );
    if rc.is_err() || ra.is_err() {
        println!("[Warning] Failed to save data");
    }
}

fn load() -> Result<(Vec<Order>, Vec<Order>), Box<dyn Error>> {
    let current = fs::read_to_string("./current.json").unwrap_or_else(|_| "[]".to_owned());
    let current = serde_json::from_str::<Vec<Order>>(current.as_str())?;
    let all = fs::read_to_string("./all.json").unwrap_or_else(|_| "[]".to_owned());
    let all = serde_json::from_str::<Vec<Order>>(all.as_str())?;
    Ok((current, all))
}
