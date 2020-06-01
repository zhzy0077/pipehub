use serde::Serializer;

// Serde crate enforces following signature.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn bool_to_int<S>(input: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(if *input { 1 } else { 0 })
}
