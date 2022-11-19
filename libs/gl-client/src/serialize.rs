use prost::{self, Message};
use anyhow::Result;

/// CertFile is used to serialize the certificates that are used to
/// configure the tls connection into a single blob that can be stored
/// outside of the gl-client.
#[derive(Clone, PartialEq, Message)]
pub struct CertFile {
    #[prost(bytes, tag = "1")]
    pub cert: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub key: Vec<u8>,
    #[prost(bytes, tag = "3")]
    pub ca: Vec<u8>,
}

impl CertFile {
    pub fn serialize(&self) -> Result<Vec<u8>>{
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(buf.clone())
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let cf = CertFile::decode(data)?;
        Ok(cf)
    }
}

#[cfg(test)]
mod serializer_tests {
    use super::{CertFile};
    use anyhow::Result;
    

    #[test]
    fn serialize() -> Result<()> {
        let cert: Vec<u8> = vec![99, 98];
        let key = vec![97, 96];
        let ca = vec![95, 94];
        let cf = CertFile {
            cert: cert.clone(),
            key: key.clone(),
            ca: ca.clone(),
        };
        let buf = cf.serialize()?;
        for n in cert {
            assert!(buf.contains(&n));
        }
        for n in key {
            assert!(buf.contains(&n));
        }
        for n in ca {
            assert!(buf.contains(&n));
        }
        Ok(())
    }

    #[test]
    fn deserialize() -> Result<()> {
        let data = vec![10, 2, 99, 98, 18, 2, 97, 96, 26, 2, 95, 94];
        let cf = CertFile::deserialize(&data)?;
        assert!(cf.cert == vec![99, 98]);
        assert!(cf.key == vec![97, 96]);
        assert!(cf.ca == vec![95, 94]);
        Ok(())
    }
}
