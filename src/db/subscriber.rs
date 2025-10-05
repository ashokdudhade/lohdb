use crate::Result;
use crossbeam::channel::{self, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeEvent {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

pub type Subscriber = Arc<dyn Fn(ChangeEvent) + Send + Sync>;

pub struct SubscriptionHandle {
    id: Uuid,
    _sender: Sender<()>, // Used to signal shutdown
}

impl SubscriptionHandle {
    pub fn id(&self) -> Uuid {
        self.id
    }
}

pub struct EventBus {
    subscribers: Vec<(Uuid, Sender<ChangeEvent>)>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }
    
    pub fn subscribe<F>(&mut self, callback: F) -> Result<SubscriptionHandle>
    where
        F: Fn(ChangeEvent) + Send + Sync + 'static,
    {
        let id = Uuid::new_v4();
        let (tx, rx): (Sender<ChangeEvent>, Receiver<ChangeEvent>) = channel::unbounded();
        let (shutdown_tx, shutdown_rx) = channel::bounded(1);
        
        // Store the sender for this subscriber
        self.subscribers.push((id, tx));
        
        // Spawn a thread to handle events for this subscriber
        thread::spawn(move || {
            loop {
                crossbeam::select! {
                    recv(rx) -> event => {
                        match event {
                            Ok(event) => callback(event),
                            Err(_) => break, // Channel closed
                        }
                    }
                    recv(shutdown_rx) -> _ => {
                        break; // Shutdown signal received
                    }
                }
            }
        });
        
        Ok(SubscriptionHandle {
            id,
            _sender: shutdown_tx,
        })
    }
    
    pub fn publish(&self, event: ChangeEvent) -> Result<()> {
        // Send to all active subscribers
        for (_, sender) in &self.subscribers {
            // Use try_send to avoid blocking if a subscriber is slow
            let _ = sender.try_send(event.clone());
        }
        Ok(())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}