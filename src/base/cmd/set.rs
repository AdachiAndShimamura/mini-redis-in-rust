use bytes::Bytes;

pub struct Set {
    pub key: String,
    pub value: Bytes,
}

impl Set {
    pub fn new(key: &str, value: Bytes) -> Set {
        Set {
            key: key.to_string(),
            value,
        }
    }
}
