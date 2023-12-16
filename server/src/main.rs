use axum::{http::StatusCode, routing::get, Router};
use events::EventType;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};

mod client;
mod events;
mod orders;

#[tokio::main]
async fn main() {
    let (event_routes, event_sender) = events::new();
    let event_sender = Arc::new(event_sender);
    let es1 = event_sender.clone();
    let es2 = event_sender.clone();
    let routes = Router::new()
        .merge(client::client_handler(Some("index.html")))
        .nest("/events", event_routes)
        .nest("/orders", orders::routes(event_sender.clone()))
        .route(
            "/test-event",
            get(move || async move {
                event_sender(
                    EventType::ImageChange,
                    Box::new(|i| {
                        format!(
                            "{{
                                \"url\": \"https://picsum.photos/1920/{}\",
                                \"title\": \"Test\",
                                \"subtitle\": \"This is a test from the server\"
                            }}",
                            1080 + (SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis()
                                + i as u128 % 100)
                        )
                    }),
                )
                .await
            }),
        )
        .route(
            "/test-show",
            get(|| async move {
                es1(
                    EventType::PopupShow,
                    Box::new(|i| format!("Popup from Server ({})", i)),
                )
                .await
            }),
        )
        .route(
            "/test-hide",
            get(|| async move { es2(EventType::PopupHide, Box::new(|_| String::new())).await }),
        )
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    let addr = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 8080);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}
