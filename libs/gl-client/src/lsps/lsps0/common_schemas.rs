use serde::de::Error as SeError;
use serde::ser::Error as DeError;
use serde::{Deserialize, Serialize};

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
}
