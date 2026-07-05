use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DigestAlgorithm {
    Sha256,
}

impl DigestAlgorithm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest {
    pub algorithm: DigestAlgorithm,
    pub bytes: [u8; 32],
}

impl Digest {
    pub fn sha256(bytes: impl AsRef<[u8]>) -> Self {
        let digest = Sha256::digest(bytes.as_ref());
        let mut output = [0_u8; 32];
        output.copy_from_slice(&digest);
        Self {
            algorithm: DigestAlgorithm::Sha256,
            bytes: output,
        }
    }

    pub fn from_canonical_json<T: Serialize>(value: &T) -> Result<Self, HashError> {
        Ok(Self::sha256(canonical_json_bytes(value)?))
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.as_str(), hex::encode(self.bytes))
    }
}

impl FromStr for Digest {
    type Err = HashError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (algorithm, hex_bytes) = value
            .split_once(':')
            .ok_or_else(|| HashError::InvalidDigest(value.to_owned()))?;

        if algorithm != DigestAlgorithm::Sha256.as_str() {
            return Err(HashError::UnsupportedAlgorithm(algorithm.to_owned()));
        }

        let decoded =
            hex::decode(hex_bytes).map_err(|_| HashError::InvalidDigest(value.to_owned()))?;
        let bytes: [u8; 32] = decoded
            .try_into()
            .map_err(|_| HashError::InvalidDigest(value.to_owned()))?;

        Ok(Self {
            algorithm: DigestAlgorithm::Sha256,
            bytes,
        })
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DigestVisitor;

        impl<'de> Visitor<'de> for DigestVisitor {
            type Value = Digest;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sha256 digest string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Digest::from_str(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(DigestVisitor)
    }
}

pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, HashError> {
    let value = serde_json::to_value(value)?;
    let canonical = canonicalize_json(value);
    Ok(serde_json::to_vec(&canonical)?)
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(canonicalize_json).collect()),
        Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));

            let mut normalized = serde_json::Map::new();
            for (key, nested) in entries {
                normalized.insert(key, canonicalize_json(nested));
            }

            Value::Object(normalized)
        }
        scalar => scalar,
    }
}

#[derive(Debug, Error)]
pub enum HashError {
    #[error("invalid digest: {0}")]
    InvalidDigest(String),
    #[error("unsupported digest algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("failed to serialize canonical JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::{Digest, canonical_json_bytes};
    use serde_json::json;

    #[test]
    fn canonical_json_sorts_object_keys() {
        let left = json!({
            "b": 2,
            "a": {
                "d": 4,
                "c": 3
            }
        });
        let right = json!({
            "a": {
                "c": 3,
                "d": 4
            },
            "b": 2
        });

        assert_eq!(
            canonical_json_bytes(&left).unwrap(),
            canonical_json_bytes(&right).unwrap()
        );
        assert_eq!(
            Digest::from_canonical_json(&left).unwrap(),
            Digest::from_canonical_json(&right).unwrap()
        );
    }
}
