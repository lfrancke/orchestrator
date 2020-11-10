use crate::models::{BaseResource, GroupResourceType};

use std::pin::Pin;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::time::Duration;

use actix_web::Error;
use bytes::Bytes;
use futures::Stream;
use serde::Serialize;
use std::collections::HashMap;


/// Variants of this enum will be returned for all _watch_ requests.
/// Kubernetes has the additional BOOKMARK and ERROR Types which are not implemented here yet.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "object", rename_all = "UPPERCASE")]
pub enum WatchEvent {
    ADDED(BaseResource),
}

#[derive(Debug)]
pub struct WrappedWatchEvent {
    event: WatchEvent,
    grt: GroupResourceType
}

impl WrappedWatchEvent {
    pub fn new(event: WatchEvent, grt: GroupResourceType) -> WrappedWatchEvent {
        WrappedWatchEvent {
            event,
            grt
        }
    }
}

/// The EventBroker is used to ferry events around the system.
/// It gets notified about changes (e.g. new objects) and forwards those to interested parties.
pub struct EventBroker {
    // TODO: This can be changed to RwLock<HashMap<GroupResourceType, RwLock<Vec<Sender<WatchEvent>>>>> instead (or similar)
    observers: Mutex<HashMap<GroupResourceType, Vec<Sender<WatchEvent>>>>,
}

impl EventBroker {
    pub fn new() -> Self {
        Self {
            observers: Mutex::new(HashMap::new()),
        }
    }

    /// A new event can be posted here
    // TODO: This should almost certainly be asynchronous so anything that posts a new event doesn't
    // have to wait for all watchers to be notified
    pub fn new_event(&self, event: WrappedWatchEvent) {
        let mut observers = self.observers.lock().unwrap();
        println!(
            "Received new event, sending to [{}] observers: {:?}",
            observers.len(),
            event
        );

        let mut grt_observers = observers.entry(event.grt).or_default();
        let event = event.event;

        grt_observers.iter().enumerate().for_each(|(idx, obs)| {
            let send_result = obs.send(event.clone());
            match send_result {
                Err(_) => {
                    println!("Error sending, removing observer");
                    // TODO: Need to remove observer, this doesn't work yet:
                    //grt_observers.swap_remove(idx);
                }
                Ok(_) => {}
            }
        });
    }

    pub fn register(&self, (grt, observer): (GroupResourceType, Sender<WatchEvent>)) {
        let mut observers = self.observers.lock().unwrap();
        observers.entry(grt).or_default().push(observer);
    }
}

// This is the stream that will get notified about new events
// There will be one for each running long-poll
pub struct WatchStream {
    receiver: Receiver<WatchEvent>,
}

impl WatchStream {
    pub fn new(receiver: Receiver<WatchEvent>) -> Self {
        Self { receiver }
    }
}

impl Drop for WatchStream {
    fn drop(&mut self) {
        println!("Drop stream");
    }
}

impl Stream for WatchStream {
    type Item = Result<Bytes, Error>;

    // TODO: The context has a "Waker" which needs to be passed to the EventBroker so it can wake up the required threads
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = self.receiver.recv_timeout(Duration::from_secs(1));
        return match result {
            Ok(x) => {
                println!("Received new watch event {:?}", x);
                let mut json = serde_json::to_vec(&x).unwrap();
                let mut newline = "\n".as_bytes().to_vec();
                json.append(&mut newline);
                Poll::Ready(Some(Ok(Bytes::from(json))))
            }
            Err(_) => Poll::Pending,
        }
    }
}
