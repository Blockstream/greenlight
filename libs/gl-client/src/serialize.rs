use prost::Message;
use std::convert::{TryFrom, TryInto};

/// AuthBlob is used to serialize the certificates that are used to
/// configure the tls connection, and the rune that is used to
/// authorize requests, into a single blob that can be stored outside
/// of the gl-client.
#[derive(Clone, Message)]
pub struct AuthBlob {
    #[prost(bytes, tag = "1")]
    pub cert: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub key: Vec<u8>,
    #[prost(bytes, tag = "3")]
    pub ca: Vec<u8>,
    #[prost(string, tag = "4")]
    pub rune: String,
}

impl TryFrom<&[u8]> for AuthBlob {
    type Error = prost::DecodeError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        AuthBlob::decode(data)
    }
}

impl TryInto<Vec<u8>> for AuthBlob {
    type Error = prost::EncodeError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(buf.clone())
    }
}

impl AuthBlob {
    pub fn serialize(self) -> anyhow::Result<Vec<u8>> {
        let data: Vec<u8> = self.try_into()?;
        Ok(data)
    }

    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let blob = AuthBlob::try_from(data)?;
        Ok(blob)
    }
}

#[cfg(test)]
mod serializer_tests {
    use super::AuthBlob;

    #[test]
    fn serialize() {
        let cert: Vec<u8> = vec![99, 98];
        let key = vec![97, 96];
        let ca = vec![95, 94];
        let rune = "non_functional_rune".to_string();
        let blob = AuthBlob {
            cert: cert.clone(),
            key: key.clone(),
            ca: ca.clone(),
            rune: rune.clone(),
        };
        let buf: Vec<u8> = blob.serialize().unwrap();
        for n in cert {
            assert!(buf.contains(&n));
        }
        for n in key {
            assert!(buf.contains(&n));
        }
        for n in ca {
            assert!(buf.contains(&n));
        }
        for n in rune.as_bytes() {
            assert!(buf.contains(n));
        }
    }

    #[test]
    fn deserialize() {
        let data = vec![
            10, 2, 99, 98, 18, 2, 97, 96, 26, 2, 95, 94, 34, 19, 110, 111, 110, 95, 102, 117, 110,
            99, 116, 105, 111, 110, 97, 108, 95, 114, 117, 110, 101,
        ];
        let blob = AuthBlob::deserialize(&data).unwrap();
        assert!(blob.cert == vec![99, 98]);
        assert!(blob.key == vec![97, 96]);
        assert!(blob.ca == vec![95, 94]);
        assert!(blob.rune == *"non_functional_rune");
    }
}
