use futures::{executor::block_on, lock::Mutex};
use std::{collections::VecDeque, sync::Arc};
use tokio::time::Interval;

use crate::events::{EventAddFunction, EventType};

pub struct Popups {
    popups: Arc<Mutex<VecDeque<String>>>,
    show_timeout: Arc<Mutex<Interval>>,
    wait_timeout: Arc<Mutex<Interval>>,
    add_event: Arc<EventAddFunction>,
}

impl Popups {
    pub fn new(add_event: Arc<EventAddFunction>, show_seconds: u64, wait_seconds: u64) -> Self {
        Self {
            popups: Arc::new(Mutex::new(VecDeque::new())),
            show_timeout: Arc::new(Mutex::new(tokio::time::interval(
                tokio::time::Duration::from_secs(show_seconds),
            ))),
            wait_timeout: Arc::new(Mutex::new(tokio::time::interval(
                tokio::time::Duration::from_secs(wait_seconds),
            ))),
            add_event,
        }
    }

    pub fn add_popup(&mut self, popup: String) {
        {
            let mut popups = block_on(self.popups.lock());
            popups.push_back(popup);
            drop(popups);
        };
    }

    pub fn set_show_timeout(&mut self, seconds: u64) {
        *block_on(self.show_timeout.lock()) =
            tokio::time::interval(tokio::time::Duration::from_secs(seconds));
    }

    pub fn set_wait_timeout(&mut self, seconds: u64) {
        *block_on(self.wait_timeout.lock()) =
            tokio::time::interval(tokio::time::Duration::from_secs(seconds));
    }

    pub fn run(&self) -> tokio::task::JoinHandle<()> {
        let popups = self.popups.clone();
        let show_timeout = self.show_timeout.clone();
        let wait_timeout = self.wait_timeout.clone();
        let add_event = self.add_event.clone();
        tokio::task::spawn(async move {
            loop {
                {
                    let mut wait_timeout = wait_timeout.lock().await;
                    wait_timeout.reset();
                    wait_timeout.tick().await;
                };
                {
                    let popup = popups.lock().await.pop_front();
                    // let popup = popups.;
                    if let Some(popup) = popup {
                        let _ =
                            add_event(EventType::PopupShow, Box::new(move |_| popup.clone())).await;
                    } else {
                        // Wait for next popup
                        continue;
                    }
                };
                {
                    let mut show_timeout = show_timeout.lock().await;
                    show_timeout.reset();
                    show_timeout.tick().await;
                };
                {
                    let _ = add_event(EventType::PopupHide, Box::new(move |_| String::new())).await;
                };
            }
        })
    }
}
