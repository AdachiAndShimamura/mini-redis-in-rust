use bytes::Bytes;

pub struct Publish {
    pub(crate) key: String,
    pub(crate) value: Bytes,
}
