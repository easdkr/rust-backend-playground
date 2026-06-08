use std::collections::HashMap;
use std::sync::Arc;

use application::notification::dto::NotificationDto;
use application::notification::service::NotificationBroadcaster;
use tokio::sync::{Mutex, broadcast};

pub type NotificationSender = broadcast::Sender<NotificationDto>;
pub type NotificationReceiver = broadcast::Receiver<NotificationDto>;

#[derive(Clone)]
pub struct InMemoryNotificationBroadcaster {
    channels: Arc<Mutex<HashMap<String, NotificationSender>>>,
    capacity: usize,
}

impl InMemoryNotificationBroadcaster {
    pub fn new(capacity: usize) -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    pub async fn subscribe(&self, user_id: &str) -> NotificationReceiver {
        let mut channels = self.channels.lock().await;
        let sender = channels
            .entry(user_id.to_string())
            .or_insert_with(|| broadcast::channel(self.capacity).0);
        sender.subscribe()
    }

    pub async fn unsubscribe(&self, user_id: &str) {
        let mut channels = self.channels.lock().await;
        if let Some(sender) = channels.get(user_id) {
            if sender.receiver_count() == 0 {
                channels.remove(user_id);
            }
        }
    }
}

#[async_trait::async_trait]
impl NotificationBroadcaster for InMemoryNotificationBroadcaster {
    async fn broadcast(&self, user_id: &str, notification: &NotificationDto) {
        let channels = self.channels.lock().await;
        if let Some(sender) = channels.get(user_id) {
            let _ = sender.send(notification.clone());
        }
    }
}
