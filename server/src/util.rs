use serde::{Deserialize, Deserializer, Serializer};
use std::ops::Add;
use std::time::{Duration, Instant};

// Serde crate enforces following signature.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn bool_to_int<S>(input: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(if *input { 1 } else { 0 })
}

pub fn expires_at<'de, D>(deserializer: D) -> Result<Instant, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer)
        .map(|expires_in| Instant::now().add(Duration::from_secs(expires_in)))
}
