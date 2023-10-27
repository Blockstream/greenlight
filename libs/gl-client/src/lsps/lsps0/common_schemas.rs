use core::str::FromStr;

use std::fmt::{Display, Formatter};

use anyhow::{anyhow, Context};

use serde::de::Error as SeError;
use serde::ser::Error as DeError;
use serde::{Deserialize, Serialize, Deserializer, Serializer};

use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};

// Implements all the common schema's defined in LSPS0 common schema's

// Initially I used serde_as for the parsing and serialization of this type.
// However, the spec is more strict.
// It requires a yyyy-mm-ddThh:mm:ss.uuuZ format
//
// The serde_as provides us options such as rfc_3339.
// Note, that this also allows formats that are not compliant to the LSP-spec such as dropping
// the fractional seconds or use non UTC timezones.
//
// For LSPS2 the `valid_until`-field must be copied verbatim. As a client this can only be
// achieved if the LSPS2 sends a fully compliant timestamp.
//
// I have decided to fail early if another timestamp is received
#[derive(Debug)]
pub struct IsoDatetime {
    pub datetime: PrimitiveDateTime,
}

const DATETIME_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

impl IsoDatetime {
    pub fn from_offset_date_time(datetime: OffsetDateTime) -> Self {
        let offset = time::UtcOffset::from_whole_seconds(0).unwrap();
        let datetime_utc = datetime.to_offset(offset);
        let primitive = PrimitiveDateTime::new(datetime_utc.date(), datetime.time());
        Self {
            datetime: primitive,
        }
    }

    pub fn from_primitive_date_time(datetime: PrimitiveDateTime) -> Self {
        Self { datetime }
    }

    pub fn datetime(&self) -> OffsetDateTime {
        self.datetime.assume_utc()
    }
}

impl Serialize for IsoDatetime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let datetime_str = self
            .datetime
            .format(&DATETIME_FORMAT)
            .map_err(|err| S::Error::custom(format!("Failed to format datetime {:?}", err)))?;

        serializer.serialize_str(&datetime_str)
    }
}

impl<'de> Deserialize<'de> for IsoDatetime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        time::PrimitiveDateTime::parse(&str_repr, DATETIME_FORMAT)
            .map_err(|err| D::Error::custom(format!("Failed to parse Datetime. {:?}", err)))
            .map(Self::from_primitive_date_time)
    }
}

#[derive(Debug)]
pub struct SatAmount(u64);
#[derive(Debug)]
pub struct MsatAmount(u64);

impl SatAmount {
    pub fn sat_value(&self) -> u64 {
        self.0
    }

    pub fn new(value: u64) -> Self {
        SatAmount(value)
    }
}

impl MsatAmount {
    pub fn msat_value(&self) -> u64 {
        self.0
    }

    pub fn new(value: u64) -> Self {
        MsatAmount(value)
    }
}

impl Serialize for SatAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let amount_str = self.0.to_string();
        serializer.serialize_str(&amount_str)
    }
}

impl Serialize for MsatAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let amount_str = self.0.to_string();
        serializer.serialize_str(&amount_str)
    }
}

impl<'de> Deserialize<'de> for SatAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        let u64_repr: Result<u64, _> = str_repr
            .parse()
            .map_err(|_| D::Error::custom(String::from("Failed to parse sat_amount")));
        Ok(Self(u64_repr.unwrap()))
    }
}

impl<'de> Deserialize<'de> for MsatAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        let u64_repr: Result<u64, _> = str_repr
            .parse()
            .map_err(|_| D::Error::custom(String::from("Failed to parse sat_amount")));
        Ok(Self(u64_repr.unwrap()))
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortChannelId(u64);

impl Serialize for ShortChannelId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ShortChannelId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(&s).map_err(|e| Error::custom(e.to_string()))?)
    }
}

impl FromStr for ShortChannelId {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Result<Vec<u64>, _> = s.split('x').map(|p| p.parse()).collect();
        let parts = parts.with_context(|| format!("Malformed short_channel_id: {}", s))?;
        if parts.len() != 3 {
            return Err(anyhow!(
                "Malformed short_channel_id: element count mismatch"
            ));
        }

        Ok(ShortChannelId(
            (parts[0] << 40) | (parts[1] << 16) | (parts[2] << 0),
        ))
    }
}
impl Display for ShortChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}x{}", self.block(), self.txindex(), self.outnum())
    }
}
impl ShortChannelId {
    pub fn block(&self) -> u32 {
        (self.0 >> 40) as u32 & 0xFFFFFF
    }
    pub fn txindex(&self) -> u32 {
        (self.0 >> 16) as u32 & 0xFFFFFF
    }
    pub fn outnum(&self) -> u16 {
        self.0 as u16 & 0xFFFF
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_amount_sats() {
        // Pick a number which exceeds 2^32 to ensure internal representation exceeds 32 bits
        let json_str_number = "\"10000000001\"";

        let int_number: u64 = 10000000001;

        let x = serde_json::from_str::<SatAmount>(json_str_number).unwrap();
        assert_eq!(x.sat_value(), int_number);
    }

    #[test]
    fn serializing_amount_sats() {
        // Pick a number which exceeds 2^32 to ensure internal representation exceeds 32 bits
        // The json_str includes the " to indicate it is a string
        let json_str_number = "\"10000000001\"";
        let int_number: u64 = 10000000001;

        let sat_amount = SatAmount::new(int_number);

        let json_str = serde_json::to_string::<SatAmount>(&sat_amount).unwrap();
        assert_eq!(json_str, json_str_number);
    }

    #[test]
    fn parse_and_serialize_datetime() {
        let datetime_str = "\"2023-01-01T23:59:59.999Z\"";

        let dt = serde_json::from_str::<IsoDatetime>(datetime_str).unwrap();

        assert_eq!(dt.datetime.year(), 2023);
        assert_eq!(dt.datetime.month(), time::Month::January);
        assert_eq!(dt.datetime.day(), 1);
        assert_eq!(dt.datetime.hour(), 23);
        assert_eq!(dt.datetime.minute(), 59);
        assert_eq!(dt.datetime.second(), 59);

        assert_eq!(
            serde_json::to_string(&dt).expect("Can be serialized"),
            datetime_str
        )
    }

    #[test]
    fn parse_datetime_that_doesnt_follow_spec() {
        // The spec doesn't explicitly say that clients have to ignore datetimes that don't follow the spec
        // However, in LSPS2 the datetime_str must be repeated verbatim
        let datetime_str = "\"2023-01-01T23:59:59.99Z\"";

        let result = serde_json::from_str::<IsoDatetime>(datetime_str);
        result.expect_err("datetime_str should not be parsed if it doesn't follow spec");
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn parse_scid_from_string() {
        // How to read this test
        //
        // The shortchannel_id is 8 bytes long.
        // The 3 first bytes are the blockheight, 3 next bytes are the txid and last 2 bytes are vout
        // This explains the unusual byte groupings

        // The string representation are the same numbers separated by the letter x

        // Test the largest possible value
        let scid_str = "16777215x16777215x65535";

        let scid = ShortChannelId::from_str(scid_str).expect("The scid is parseable");
        assert_eq!(scid.to_string(), scid_str);

        // Test the smallest possible value
        let scid_str = "0x0x0";

        let scid = ShortChannelId::from_str(scid_str).expect("The scid is parseable");
        assert_eq!(scid.to_string(), scid_str);

        let scid_str = "1x2x3";

        let scid = ShortChannelId::from_str(scid_str).expect("The scid is parseable");
        assert_eq!(scid.to_string(), scid_str);
        // A couple of unparseable scids
        assert!(ShortChannelId::from_str("xx").is_err());
        assert!(ShortChannelId::from_str("0x0").is_err());
        assert!(ShortChannelId::from_str("-2x-12x14").is_err());
    }

    #[test]
    fn short_channel_id_is_serialized_as_str() {
        let scid: ShortChannelId = ShortChannelId::from_str("10x5x8").unwrap();
        let scid_json_obj = serde_json::to_string(&scid).expect("Can be serialized");
        assert_eq!("\"10x5x8\"", scid_json_obj);
    }

    #[test]
    fn short_channel_id_can_be_deserialized_from_str() {
        let scid_json = "\"11x12x13\"";

        let scid = serde_json::from_str::<ShortChannelId>(scid_json).expect("scid can be parsed");

        assert_eq!(scid, ShortChannelId::from_str("11x12x13").unwrap());
    }
}
