use axum::{http::StatusCode, Router};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

mod client;

#[tokio::main]
async fn main() {
    let routes = Router::new()
        .merge(client::client_handler(Some("index.html")))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") });

    let addr = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 8080);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}
