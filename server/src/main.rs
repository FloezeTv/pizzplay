use axum::{http::StatusCode, routing::get, Router};
use events::EventType;
use futures::lock::Mutex;
use images::Images;
use popups::Popups;
use std::{
    fs,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};
use tower_http::services::ServeDir;

mod args;
mod client;
mod events;
mod images;
mod orders;
mod popups;

#[tokio::main]
async fn main() {
    let args = args::parse();

    let (event_routes, event_sender) = events::new();
    let event_sender = Arc::new(event_sender);
    let mut images = Images::new(event_sender.clone(), args.image_timeout, args.image_offset);
    let image_data = fs::read_to_string(args.image_path).unwrap_or("[]".to_owned());
    images
        .set_images(&image_data)
        .expect(format!("Failed to read image data: {image_data:?}").as_str());
    images.run();
    let popups = Arc::new(Mutex::new(Popups::new(
        event_sender.clone(),
        args.popup_show,
        args.popup_wait,
    )));
    {
        popups.lock().await.run();
    };
    let es1 = event_sender.clone();
    let es2 = event_sender.clone();
    let routes = Router::new()
        .merge(client::client_handler(Some("index.html")))
        .nest("/events", event_routes)
        .nest("/orders", orders::routes(event_sender.clone(), popups))
        .nest_service("/assets", ServeDir::new(args.assets_dir))
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
