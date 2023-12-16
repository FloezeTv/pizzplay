use axum::{
    extract::{Query, State},
    response::{sse::Event, IntoResponse, Sse},
    routing::get,
    Router,
};
use enum_map::{Enum, EnumMap};
use futures::{future::BoxFuture, lock::Mutex, Stream};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt as _, StreamMap};

/// Creates a new server-sent-events system.
///
/// ## Return values
/// Returns `(router, update_fn)`, where
/// - `router`: [axum] [Router] which can be used to allow clients to subscribe to events as `/subscribe`.
///   Client will be subscribed to all events specified in query parameters.
pub fn new() -> (Router, EventAddFunction) {
    let event_channels: EventChannels =
        Arc::new(EnumMap::from_fn(|_: EventType| Mutex::new(Vec::new())));
    let router = Router::new()
        .route("/subscribe", get(subscribe))
        .with_state(event_channels.clone());
    let update_fn: EventAddFunction =
        Box::new(move |e, mut s: Box<dyn FnMut(usize) -> String + Send>| {
            let ec = event_channels.clone();
            Box::pin(async move {
                let mut channels = ec[e].lock().await;
                channels.retain(|v| !v.is_closed());
                let mut error = false;
                channels.iter().enumerate().for_each(|(idx, channel)| {
                    let r = channel.send(s(idx));
                    if r.is_err() {
                        error = true;
                    }
                });
                if error {
                    Err(())
                } else {
                    Ok(())
                }
            })
        });
    (router, update_fn)
}

#[derive(Enum, PartialEq, Eq, Debug)]
pub enum EventType {
    ImageChange,
    PopupShow,
    PopupHide,
    OrdersUpdated,
}

pub type EventAddFunction = Box<
    dyn Fn(EventType, Box<dyn FnMut(usize) -> String + Send>) -> BoxFuture<'static, Result<(), ()>>
        + Send
        + Sync,
>;

type EventChannels = Arc<EnumMap<EventType, Mutex<Vec<UnboundedSender<String>>>>>;

#[derive(Deserialize)]
struct SubscriptionTarget {
    image_change: Option<String>,
    popup_show: Option<String>,
    popup_hide: Option<String>,
    orders_updated: Option<String>,
}

async fn get_subscribed_streams(
    event_channels: &EventChannels,
    subscription_target: &SubscriptionTarget,
) -> impl Stream<Item = Event> {
    let mut streams = StreamMap::new();

    for (check, event_type, event_name) in [
        (
            &subscription_target.image_change,
            EventType::ImageChange,
            "image_change",
        ),
        (
            &subscription_target.popup_show,
            EventType::PopupShow,
            "popup_show",
        ),
        (
            &subscription_target.popup_hide,
            EventType::PopupHide,
            "popup_hide",
        ),
        (
            &subscription_target.orders_updated,
            EventType::OrdersUpdated,
            "orders_updated",
        ),
    ] {
        if check.is_some() {
            let (sender, receiver) = mpsc::unbounded_channel::<String>();
            event_channels[event_type].lock().await.push(sender);
            streams.insert(event_name, UnboundedReceiverStream::new(receiver));
        }
    }

    streams.map(|(name, value)| Event::default().event(name).data(value))
}

async fn subscribe(
    State(event_channels): State<EventChannels>,
    Query(subscription_target): Query<SubscriptionTarget>,
) -> impl IntoResponse {
    let streams = get_subscribed_streams(&event_channels, &subscription_target)
        .await
        .map(Ok::<_, String>);

    Sse::new(streams).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive-text"),
    )
}
