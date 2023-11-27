use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync;
use tokio::sync::RwLock;

#[derive(Default, Debug, Clone)]
pub struct DB {
    data: Arc<RwLock<DataBase>>,
}

impl DB {
    pub async fn get(&self, key: String) -> Option<Bytes> {
        self.data.read().await.data.get(key.as_str()).cloned()
    }

    pub async fn set(&mut self, key: String, value: Bytes) {
        self.data.write().await.set(key, value);
    }

    pub async fn delete(&mut self, key: String) {
        self.data.write().await.data.remove(key.as_str());
    }

    pub async fn subscribe(&mut self, channel: &String) -> sync::broadcast::Receiver<Bytes> {
        return if let Some(sender) = self.data.read().await.subscribers.get(channel) {
            sender.subscribe()
        } else {
            let (sender, receiver) = sync::broadcast::channel(10);
            self.data.write().await.subscribers.insert(channel.clone(), sender);
            receiver
        };
    }

    pub async fn publish(&mut self, channel: String, message: Bytes) -> Result<()> {
        return if let Some(channel) = self.data.write().await.subscribers.get(&channel) {
            channel.send(message).unwrap();
            Ok(())
        } else {
            let (sender, _) = sync::broadcast::channel(10);
            self.data.write().await.subscribers.insert(channel, sender);
            Err(anyhow!("This channel has no subscriber!"))
        };
    }
}

#[derive(Default, Debug)]
pub struct DataBase {
    data: HashMap<String, Bytes>,
    subscribers: HashMap<String, sync::broadcast::Sender<Bytes>>,

}

impl DataBase {
    pub fn get(&self, key: &String) -> Option<Bytes> {
        self.data.get(key.as_str()).cloned()
    }

    pub fn set(&mut self, key: String, value: Bytes) {
        self.data.insert(key, value);
    }

    pub fn delete(&mut self, key: &String) {
        self.data.remove(key.as_str());
    }

    pub fn subscribe(&mut self, channel: &String) -> sync::broadcast::Receiver<Bytes> {
        return if let Some(sender) = self.subscribers.get(channel) {
            sender.subscribe()
        } else {
            let (sender, receiver) = sync::broadcast::channel(10);
            self.subscribers.insert(channel.clone(), sender);
            receiver
        };
    }

    pub fn publish(&mut self, channel: &String, message: Bytes) -> Result<()> {
        return if let Some(channel) = self.subscribers.get(channel) {
            channel.send(message).unwrap();
            Ok(())
        } else {
            let (sender, _) = sync::broadcast::channel(10);
            self.subscribers.insert(channel.clone(), sender);
            Err(anyhow!("This channel has no subscriber!"))
        };
    }
}

#[derive(Default, Debug)]
pub struct Entry {
    bytes: Bytes,
}
