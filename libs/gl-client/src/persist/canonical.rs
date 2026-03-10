use anyhow::anyhow;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

pub struct CanonicalJsonValue<'a>(pub &'a serde_json::Value);

impl Serialize for CanonicalJsonValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            serde_json::Value::Null => serializer.serialize_unit(),
            serde_json::Value::Bool(v) => serializer.serialize_bool(*v),
            serde_json::Value::Number(v) => v.serialize(serializer),
            serde_json::Value::String(v) => serializer.serialize_str(v),
            serde_json::Value::Array(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(&CanonicalJsonValue(value))?;
                }
                seq.end()
            }
            serde_json::Value::Object(values) => {
                let mut entries: Vec<_> = values.iter().collect();
                entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));

                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, &CanonicalJsonValue(value))?;
                }
                map.end()
            }
        }
    }
}

pub fn canonical_json_bytes(value: &serde_json::Value) -> anyhow::Result<Vec<u8>> {
    serde_json::to_vec(&CanonicalJsonValue(value))
        .map_err(|e| anyhow!("failed to serialize canonical signer state value: {e}"))
}
