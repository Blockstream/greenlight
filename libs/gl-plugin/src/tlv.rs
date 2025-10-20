use anyhow::anyhow;
use bytes::{Buf, BufMut};
use cln_rpc::primitives::TlvEntry;
use serde::{Deserialize, Deserializer};

/// A standalone type the represent a binary serialized
/// TlvStream. This is distinct from TlvStream since that expects TLV
/// streams to be encoded as maps in JSON.
#[derive(Clone, Debug)]
pub struct SerializedTlvStream {
    entries: Vec<TlvEntry>,
}

impl SerializedTlvStream {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn get(&self, typ: u64) -> Option<TlvEntry> {
        self.entries.iter().filter(|e| e.typ == typ).next().cloned()
    }

    pub fn insert(&mut self, e: TlvEntry) -> Result<(), anyhow::Error> {
        if let Some(old) = self.get(e.typ) {
            return Err(anyhow!(
                "TlvStream contains entry of type={}, old={:?}, new={:?}",
                e.typ,
                old,
                e
            ));
        }

        self.entries.push(e);
        self.entries
            .sort_by(|a, b| a.typ.partial_cmp(&b.typ).unwrap());

        Ok(())
    }

    pub fn set_bytes<T>(&mut self, typ: u64, val: T)
    where
        T: AsRef<[u8]>,
    {
        let pos = self.entries.iter().position(|e| e.typ == typ);
        match pos {
            Some(i) => self.entries[i].value = val.as_ref().to_vec(),
            None => self
                .insert(TlvEntry {
                    typ,
                    value: val.as_ref().to_vec(),
                })
                .unwrap(),
        }
    }

    pub fn set_tu64(&mut self, typ: u64, val: TU64) {
        let mut b = bytes::BytesMut::new();
        b.put_tu64(val);
        self.set_bytes(typ, b)
    }
}

pub trait FromBytes: Sized {
    type Error;
    fn from_bytes<T>(s: T) -> Result<Self, Self::Error>
    where
        T: AsRef<[u8]> + 'static;
}

impl FromBytes for SerializedTlvStream {
    type Error = anyhow::Error;
    fn from_bytes<T>(s: T) -> Result<Self, Self::Error>
    where
        T: AsRef<[u8]> + 'static,
    {
        let mut b = s.as_ref();
        //let mut b: bytes::Bytes = r.into();
        let mut entries: Vec<TlvEntry> = vec![];
        while b.remaining() >= 2 {
            let typ = b.get_compact_size() as u64;
            let len = b.get_compact_size() as usize;
            let value = b.copy_to_bytes(len).to_vec();
            entries.push(TlvEntry { typ, value });
        }

        Ok(SerializedTlvStream { entries })
    }
}

pub type CompactSize = u64;

/// A variant of CompactSize that works on length-delimited
/// buffers and therefore does not require a length prefix
pub type TU64 = u64;

/// Extensions on top of `Buf` to include LN proto primitives
pub trait ProtoBuf: Buf {
    fn get_compact_size(&mut self) -> CompactSize {
        match self.get_u8() {
            253 => self.get_u16().into(),
            254 => self.get_u32().into(),
            255 => self.get_u64(),
            v => v.into(),
        }
        .into()
    }

    fn get_tu64(&mut self) -> TU64 {
        match self.remaining() {
            1 => self.get_u8() as u64,
            2 => self.get_u16() as u64,
            4 => self.get_u32() as u64,
            8 => self.get_u64() as u64,
            l => panic!("Unexpect TU64 length: {}", l),
        }
    }
}

impl ProtoBuf for bytes::Bytes {}
impl ProtoBuf for &[u8] {}
impl ProtoBuf for bytes::buf::Take<bytes::Bytes> {}

pub trait ProtoBufMut: bytes::BufMut {
    fn put_compact_size(&mut self, cs: CompactSize) {
        match cs as u64 {
            0..=0xFC => self.put_u8(cs as u8),
            0xFD..=0xFFFF => {
                self.put_u8(253);
                self.put_u16(cs as u16);
            }
            0x10000..=0xFFFFFFFF => {
                self.put_u8(254);
                self.put_u32(cs as u32);
            }
            v => {
                self.put_u8(255);
                self.put_u64(v);
            }
        }
    }

    fn put_tu64(&mut self, u: TU64) {
        // Fixme: (nepet) We trim leading zero bytes here as they
        // cause some problems for the cln decoder - for now. Think
        // about an appropriate solution.
        let b: Vec<u8> = u
            .to_be_bytes()
            .iter()
            .map(|x| x.clone())
            .skip_while(|&x| x == 0)
            .collect();
        self.put_slice(&b);
    }
}

impl ProtoBufMut for bytes::BytesMut {}

pub trait ToBytes: Sized {
    fn to_bytes(s: Self) -> Vec<u8>;
}

impl ToBytes for SerializedTlvStream {
    fn to_bytes(s: Self) -> Vec<u8> {
        let mut b = bytes::BytesMut::new();

        for e in s.entries.iter() {
            b.put_compact_size(e.typ);
            b.put_compact_size(e.value.len() as u64);
            b.put(&e.value[..]);
        }
        b.to_vec()
    }
}

impl<'de> Deserialize<'de> for SerializedTlvStream {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Start by reading the hex-encoded string
        let s: String = Deserialize::deserialize(deserializer)?;
        let mut b: bytes::Bytes = hex::decode(s)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?
            .into();

        // Skip the length prefix
        let l = b.get_compact_size();
        let b = b.take(l as usize); // Protect against overruns

        Self::from_bytes(b.into_inner()).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl std::fmt::Display for SerializedTlvStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = hex::encode(SerializedTlvStream::to_bytes(self.clone()));
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlv_stream() {
        let raw_hex = "fd80e9fd01046c6e62633130306e31706e6670757a677070356a73376e653571727465326d707874666d7a703638667838676a7376776b3034366c76357a76707a7832766d687478683763777364717163717a7a737871797a3576717370357a6d6b78726539686d6864617378336b75357070336a38366472337778656b78336437383363706c6161363068783870357564733971787071797367717132646c68656177796c677534346567393363766e78666e64747a646a6e647465666b726861727763746b3368783766656e67346179746e6a3277686d74716665636a7930776777396c6665727072386b686d64667771736e386d6d7a3776643565776a34756370656439787076fd80eb022742";
        let raw = hex::decode(&raw_hex).unwrap();
        let tlv_stream = SerializedTlvStream::from_bytes(raw).unwrap();
        let invoice = tlv_stream.get(33001);
        let amount_msat = tlv_stream.get(33003);
        assert!(invoice.is_some());
        assert!(amount_msat
            .is_some_and(|v| u16::from_be_bytes(v.value[..].try_into().unwrap()) == 10050));
    }
}
