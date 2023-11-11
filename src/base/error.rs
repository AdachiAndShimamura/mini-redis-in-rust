use anyhow::Error;

pub type RedisResult<T> = Result<T, Error>;
