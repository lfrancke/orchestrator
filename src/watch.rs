use actix_web::Error;
use bytes::Bytes;
use futures::Stream;
use serde::Serialize;
use std::pin::Pin;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::time::Duration;

pub trait Observer {
    fn notify(&mut self, event: &WatchEvent);
}

/// Variants of this enum will be returned for all _watch_ requests.
/// Kubernetes has the additional BOOKMARK and ERROR Types which are not implemented here yet.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "object", rename_all = "UPPERCASE")]
pub enum WatchEvent {
    ADDED,
}

// This is the struct that will get notified about new events
pub struct EventBroker {
    observers: Mutex<Vec<Sender<WatchEvent>>>,
}

impl EventBroker {
    pub fn new() -> Self {
        Self {
            observers: Mutex::new(vec![]),
        }
    }

    /// A new event can be posted here
    // TODO: This should almost certainly be asynchronous so anything that posts a new event doesn't
    // have to wait for all watchers to be notified
    pub fn new_event(&self, event: WatchEvent) {
        let result = self.observers.lock().unwrap();
        println!(
            "Received new event, sending to [{}] observers: {:?}",
            result.len(),
            event
        );
        result.iter().for_each(|obs| {
            let send_result = obs.send(WatchEvent::ADDED);
            match send_result {
                Err(_) => {
                    println!("Error sending, removing observer");
                    //self.observers.
                }
                Ok(_) => {}
            }
        });
    }

    pub fn register(&self, observer: Sender<WatchEvent>) {
        let mut foo = self.observers.lock().unwrap();
        foo.push(observer);
    }
}

// This is the stream that will get notified about new events
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
        match result {
            Ok(x) => {
                println!("Received new watch event {:?}", x);
                let mut json = serde_json::to_vec(&x).unwrap();
                let mut newline = "\n".as_bytes().to_vec();
                json.append(&mut newline);
                return Poll::Ready(Some(Ok(Bytes::from(json))));
            }
            Err(_) => return Poll::Pending,
        }
    }
}
