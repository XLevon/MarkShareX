/// Custom deserializer for `Option<Option<T>>` fields in update requests.
///
/// Serde's default behavior treats `null` as `None` for the outermost `Option`,
/// which makes it impossible to distinguish "field not present" (don't update)
/// from "field present with null" (set to null).
///
/// This function wraps the deserialized value in `Some`, so:
/// - Missing field (handled by `#[serde(default)]`) → `None`
/// - `null` → `Some(None)` (set to null)
/// - value → `Some(Some(value))` (set to value)
pub mod double_option {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Some(Option::deserialize(deserializer)?))
    }
}
