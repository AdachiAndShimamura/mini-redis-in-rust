use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;
use tokio::sync::{OnceCell, RwLock};

pub static DATABASE: OnceCell<Arc<RwLock<HashMap<String, Bytes>>>> = OnceCell::new();

pub fn get_data_base() -> Arc<RwLock<HashMap<String, Bytes>>> {
    DATABASE.get_or_init(||{
        Arc::new(RwLock::new(HashMap::new()))
    })
}