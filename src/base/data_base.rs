use bytes::Bytes;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub static DATABASE: OnceCell<Arc<RwLock<HashMap<String, Bytes>>>> = OnceCell::new();

pub fn get_data_base() -> &'static Arc<RwLock<HashMap<String, Bytes>>> {
    DATABASE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}
