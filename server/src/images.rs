use futures::{executor::block_on, lock::Mutex};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{ops::AddAssign, sync::Arc};
use tokio::{sync::RwLock, time::Interval};

use crate::events::{EventAddFunction, EventType};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Image {
    url: String,
    title: String,
    subtitle: String,
}

pub struct Images {
    images: Arc<RwLock<Vec<Image>>>,
    timeout: Arc<Mutex<Interval>>,
    offset: Arc<Mutex<usize>>,
    current_index: Arc<Mutex<usize>>,
    add_event: Arc<EventAddFunction>,
}

impl Images {
    pub fn new(add_event: Arc<EventAddFunction>, timeout: u64, offset: usize) -> Self {
        Self {
            images: Arc::new(RwLock::new(Vec::new())),
            // images: Arc::new(RwLock::new()),
            timeout: Arc::new(Mutex::new(tokio::time::interval(
                tokio::time::Duration::from_secs(timeout),
            ))),
            offset: Arc::new(Mutex::new(offset)),
            current_index: Arc::new(Mutex::new(0)),
            add_event,
        }
    }

    pub fn set_images(&mut self, images: &str) -> serde_json::Result<()> {
        let images = serde_json::from_str(images)?;
        *block_on(self.images.write()) = images;
        Ok(())
    }

    pub fn set_offset(&mut self, offset: usize) {
        *block_on(self.offset.lock()) = offset;
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        *block_on(self.timeout.lock()) =
            tokio::time::interval(tokio::time::Duration::from_secs(seconds));
    }

    pub fn run(&self) -> tokio::task::JoinHandle<()> {
        let images = self.images.clone();
        let timeout = self.timeout.clone();
        let offset = self.offset.clone();
        let current_index = self.current_index.clone();
        let add_event = self.add_event.clone();
        tokio::task::spawn(async move {
            loop {
                {
                    timeout.lock().await.tick().await
                };
                {
                    let images = (*images.read().await).clone();
                    let offset = offset.lock().await.clone();
                    let current = current_index.lock().await.clone();
                    let _ = add_event(
                        EventType::ImageChange,
                        Box::new(move |i| {
                            get_serialized_image(&images, current, offset, i).unwrap_or_else(|_| {
                                println!("[Warning] Failed to serialize image");
                                String::new()
                            })
                        }),
                    )
                    .await;
                    current_index.lock().await.add_assign(1);
                };
            }
        })
    }
}

fn get_serialized_image(
    images: &Vec<Image>,
    current: usize,
    offset: usize,
    index: usize,
) -> serde_json::Result<String> {
    if images.is_empty() {
        return Ok(String::new());
    }
    let idx = (current + index * offset) % images.len();
    let image = &images[idx];
    serde_json::to_string(image)
}
