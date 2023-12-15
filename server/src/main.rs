use axum::{http::StatusCode, Router, routing::get};
use events::EventType;
use std::{net::{IpAddr, Ipv6Addr, SocketAddr}, sync::Arc, time::SystemTime};

mod client;
mod events;

#[tokio::main]
async fn main() {
    let (event_routes, event_sender) = events::new();
    let event_sender = Arc::new(event_sender);
    let routes = Router::new()
        .merge(client::client_handler(Some("index.html")))
        .nest("/events", event_routes)
        .route("/test-event", get(move || async move {event_sender(EventType::ImageChange, format!("{{
            \"url\": \"https://picsum.photos/1920/{}\",
            \"title\": \"Test\",
            \"subtitle\": \"This is a test from the server\"
          }}", 1080 + (SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() % 100)))}))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    let addr = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 8080);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}
