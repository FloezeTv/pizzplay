use axum::{http::StatusCode, Router, routing::get};
use events::EventType;
use std::{net::{IpAddr, Ipv6Addr, SocketAddr}, sync::Arc, time::SystemTime};

mod client;
mod events;

#[tokio::main]
async fn main() {
    let (event_routes, event_sender) = events::new();
    let event_sender = Arc::new(event_sender);
    let es1 = event_sender.clone();
    let es2 = event_sender.clone();
    let routes = Router::new()
        .merge(client::client_handler(Some("index.html")))
        .nest("/events", event_routes)
        .route("/test-event", get(move || async move {event_sender(EventType::ImageChange, format!("{{
            \"url\": \"https://picsum.photos/1920/{}\",
            \"title\": \"Test\",
            \"subtitle\": \"This is a test from the server\"
          }}", 1080 + (SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() % 100)))}))
          .route("/test-show", get(|| async move {es1(EventType::PopupShow, "Popup from Server".to_owned())}))
          .route("/test-hide", get(|| async move {es2(EventType::PopupHide, String::new())}))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    let addr = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 8080);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}
