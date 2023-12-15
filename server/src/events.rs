use axum::{
    extract::{Query, State},
    response::{sse::Event, IntoResponse, Sse},
    routing::get,
    Router,
};
use enum_map::{Enum, EnumMap};
use futures::Stream;
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    StreamExt as _, StreamMap,
};

/// Creates a new server-sent-events system.
///
/// ## Return values
/// Returns `(router, update_fn)`, where
/// - `router`: [axum] [Router] which can be used to allow clients to subscribe to events as `/subscribe`.
///   Client will be subscribed to all events specified in query parameters.
pub fn new() -> (
    Router,
    Box<dyn Fn(EventType, String) -> Result<(), ()> + Send + Sync>,
) {
    let event_channels: EventChannels = Arc::new(EnumMap::from_fn(|_: EventType| {
        Arc::new(broadcast::channel::<String>(8).0)
    }));
    let router = Router::new()
        .route("/subscribe", get(subscribe))
        .with_state(event_channels.clone());
    let send = move |c: &Sender<String>, s: String| {
        if c.receiver_count() > 0 {
            c.send(s).map(|_| ()).map_err(|_| ())
        } else {
            Ok(())
        }
    };
    let update_fn = move |e, s| send(event_channels[e].as_ref(), s);
    (router, Box::new(update_fn))
}

#[derive(Enum, PartialEq, Eq, Debug)]
pub enum EventType {
    ImageChange,
}

type EventChannels = Arc<EnumMap<EventType, Arc<Sender<String>>>>;

#[derive(Deserialize)]
struct SubscriptionTarget {
    image_change: Option<String>,
}

fn get_subscribed_streams(
    event_channels: &EventChannels,
    subscription_target: &SubscriptionTarget,
) -> impl Stream<Item = Result<Event, BroadcastStreamRecvError>> {
    let mut streams = StreamMap::new();

    if subscription_target.image_change.is_some() {
        streams.insert(
            "image_change",
            BroadcastStream::new(event_channels[EventType::ImageChange].subscribe()),
        );
    }

    streams.map(|(name, value)| value.map(|v| Event::default().event(name).data(v)))
}

async fn subscribe(
    State(event_channels): State<EventChannels>,
    Query(subscription_target): Query<SubscriptionTarget>,
) -> impl IntoResponse {
    let streams = get_subscribed_streams(&event_channels, &subscription_target);

    Sse::new(streams).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive-text"),
    )
}
