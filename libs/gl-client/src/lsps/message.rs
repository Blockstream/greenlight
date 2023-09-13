// TODO: Implement parsing of error types for lsps2.getinfo
use crate::lsps::error::LspsError;
pub use crate::lsps::json_rpc::{JsonRpcMethod, NoParams};
use crate::lsps::json_rpc_erased::{JsonRpcMethodErased, JsonRpcMethodUnerased};
use serde::de::Error as DeError;
use serde::ser::Error as SeError;
use serde::{Deserialize, Serialize};

use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;

type OnchainFeeRate = u64;

// All rpc-methods defined in the LSPS standard
// The generics are <I,O,E> where
// - I represents the params
// - O represents the result data
// - E represents the error if present
//
// To create language bindings for a new rpc-call you must
// 1. Add it to the JsonRpcMethodEnum
// 2. Add it to the from_method_name function
// 3. Add it to the ref_erase function
pub type Lsps0ListProtocols = JsonRpcMethod<NoParams, ProtocolList, ()>;
pub type Lsps1Info = JsonRpcMethod<NoParams, Lsps1InfoResponse, ()>;
pub type Lsps1Order = JsonRpcMethod<Lsps1GetOrderRequest, Lsps1GetOrderResponse, ()>;
pub type Lsps2GetVersions = JsonRpcMethod<NoParams, Lsps2GetVersionsResponse, ()>;
pub type Lsps2GetInfo = JsonRpcMethod<Lsps2GetInfoRequest, Lsp2GetInfoResponse, ()>;
pub type Lsps2Buy = JsonRpcMethod<Lsps2BuyRequest, Lsps2BuyResponse, ()>;


pub const LSPS0_LIST_PROTOCOLS: Lsps0ListProtocols =
    Lsps0ListProtocols::new("lsps0.list_protocols");

// LSPS1: Buy Channels
pub const LSPS1_GETINFO: Lsps1Info = Lsps1Info::new("lsps1.info");
pub const LSPS1_GETORDER: Lsps1Order = Lsps1Order::new("lsps1.order");

// LSPS2: JIT-channels
pub const LSPS2_GET_VERSIONS: Lsps2GetVersions = Lsps2GetVersions::new("lsps2.get_versions");
pub const LSPS2_GET_INFO: Lsps2GetInfo = Lsps2GetInfo::new("lsps2.get_info");
pub const LSPS2_BUY: Lsps2Buy = Lsps2Buy::new("lsps2.buy");

const MAX_PROMISE_LEN_BYTES: usize = 512;
#[derive(Debug)]
struct Promise {
    promise: String,
}

impl Promise {
    fn new(promise: String) -> Result<Self, String> {
        if promise.len() <= MAX_PROMISE_LEN_BYTES {
            Ok(Promise { promise: promise })
        } else {
            Err(String::from("Promise exceeds maximum length"))
        }
    }
}

impl Serialize for Promise {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return serializer.serialize_str(&self.promise);
    }
}

// We only accept promises with a max length of 512 to be compliant to the spec
// Note, that serde-json still forces us to parse the entire string fully.
// However, the client will not story the overly large json-file
impl<'de> Deserialize<'de> for Promise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = String::deserialize(deserializer)?;
        let promise = Promise::new(str_repr.clone());
        format!("{:?}", &promise);
        return Promise::new(str_repr.clone())
            .map_err(|_| D::Error::custom("promise exceeds max length"));
    }
}

pub enum JsonRpcMethodEnum {
    Lsps0ListProtocols(Lsps0ListProtocols),
    Lsps1Info(Lsps1Info),
    Lsps1Order(Lsps1Order),
    Lsp2GetVersions(Lsps2GetVersions),
    Lsps2GetInfo(Lsps2GetInfo),
    Lsps2Buy(Lsps2Buy)
}

impl JsonRpcMethodEnum {
    pub fn from_method_name(value: &str) -> Result<JsonRpcMethodEnum, LspsError> {
        match value {
            "lsps0.list_protocols" => Ok(Self::Lsps0ListProtocols(LSPS0_LIST_PROTOCOLS)),
            "lsps1.info" => Ok(Self::Lsps1Info(LSPS1_GETINFO)),
            "lsps1.order" => Ok(Self::Lsps1Order(LSPS1_GETORDER)),
            "lsps2.get_versions" => Ok(Self::Lsp2GetVersions(LSPS2_GET_VERSIONS)),
            "lsps2.get_info" => Ok(Self::Lsps2GetInfo(LSPS2_GET_INFO)),
            "lsps2.buy" => Ok(Self::Lsps2Buy(LSPS2_BUY)),
            default => Err(LspsError::MethodUnknown(String::from(default))),
        }
    }

    // Useful for language bindings.
    // The python code can
    pub fn ref_erase(&self) -> &dyn JsonRpcMethodErased {
        match self {
            Self::Lsps0ListProtocols(list_protocol) => list_protocol.ref_erase(),
            Self::Lsps1Info(info) => info.ref_erase(),
            Self::Lsps1Order(order) => order.ref_erase(),
            Self::Lsp2GetVersions(order) => order.ref_erase(),
            Self::Lsps2GetInfo(order) => order.ref_erase(),
            Self::Lsps2Buy(buy) => buy.ref_erase()
        }
    }
}

impl<'a> JsonRpcMethodUnerased<'a, Vec<u8>, Vec<u8>, Vec<u8>> for JsonRpcMethodEnum {
    fn name(&self) -> &str {
        self.ref_erase().name()
    }

    fn create_request(
        &self,
        params: Vec<u8>,
        json_rpc_id: String,
    ) -> Result<super::json_rpc::JsonRpcRequest<Vec<u8>>, serde_json::Error> {
        self.ref_erase().create_request(params, json_rpc_id)
    }

    fn parse_json_response_str(
        &self,
        json_str: &str,
    ) -> Result<super::json_rpc::JsonRpcResponse<Vec<u8>, Vec<u8>>, serde_json::Error> {
        self.ref_erase().parse_json_response_str(json_str)
    }

    fn parse_json_response_value(
        &self,
        json_value: serde_json::Value,
    ) -> Result<super::json_rpc::JsonRpcResponse<Vec<u8>, Vec<u8>>, serde_json::Error> {
        self.ref_erase().parse_json_response_value(json_value)
    }
}

#[derive(Debug)]
pub struct SatAmount(u64);
#[derive(Debug)]
pub struct MsatAmount(u64);

impl SatAmount {
    pub fn value(&self) -> u64 {
        return self.0;
    }
}

impl MsatAmount {
    pub fn value(&self) -> u64 {
        return self.0;
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
        return Ok(Self(u64_repr.unwrap()));
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
        return Ok(Self(u64_repr.unwrap()));
    }
}

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
pub struct Datetime {
    datetime: PrimitiveDateTime,
}

const DATETIME_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

impl Datetime {
    pub fn from_offset_date_time(datetime: OffsetDateTime) -> Self {
        let offset = time::UtcOffset::from_whole_seconds(0).unwrap();
        let datetime_utc = datetime.to_offset(offset);
        let primitive = PrimitiveDateTime::new(datetime_utc.date(), datetime.time());
        return Self {
            datetime: primitive,
        };
    }

    pub fn from_primitive_date_time(datetime: PrimitiveDateTime) -> Self {
        return Self { datetime: datetime };
    }

    pub fn datetime(&self) -> OffsetDateTime {
        return self.datetime.assume_utc();
    }
}

impl Serialize for Datetime {
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

impl<'de> Deserialize<'de> for Datetime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = <String as serde::de::Deserialize>::deserialize(deserializer)?;
        time::PrimitiveDateTime::parse(&str_repr, DATETIME_FORMAT)
            .map_err(|err| D::Error::custom(format!("Failed to parse Datetime. {:?}", err)))
            .map(|dt| Self::from_primitive_date_time(dt))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolList {
    pub protocols: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1InfoResponse {
    supported_versions: Vec<u16>,
    website: Option<String>,
    options: Lsps1Options,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1Options {
    minimum_channel_confirmations: Option<u8>,
    minimum_onchain_payment_confirmations: Option<u8>,
    supports_zero_channel_reserve: bool,
    min_onchain_payment_size_sat: Option<u64>,
    max_channel_expiry_blocks: Option<u32>,
    min_initial_client_balance_sat: Option<u64>,
    min_initial_lsp_balance_sat: Option<u64>,
    max_initial_client_balance_sat: Option<u64>,
    min_channel_balance_sat: Option<u64>,
    max_channel_balance_sat: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1GetOrderRequest {
    pub api_version: u16,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    pub refund_onchain_address: Option<String>,
    #[serde(rename = "announceChannel")]
    pub announce_channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1GetOrderResponse {
    uuid: Uuid,
    api_version: u16,
    lsp_balance_sat: SatAmount,
    client_balance_sat: SatAmount,
    confirms_within_blocks: u8,
    channel_expiry_blocks: u8,
    token: String,
    #[serde(rename = "announceChannel")]
    announce_channel: bool,
    created_at: Datetime,
    expires_at: Datetime,
    order_state: OrderState,
    payment: Payment,
}

#[derive(Debug, Serialize, Deserialize)]
enum OrderState {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
enum PaymentState {
    #[serde(rename = "EXPECT_PAYMENT")]
    ExpectPayment,
    #[serde(rename = "HOLD")]
    Hold,
    #[serde(rename = "STATE")]
    State,
    #[serde(rename = "REFUNDED")]
    Refunded,
}

#[derive(Debug, Serialize, Deserialize)]
struct OnchainPayment {
    outpoint: String,
    sat: SatAmount,
    confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Payment {
    state: PaymentState,
    fee_total_sat: SatAmount,
    order_total_sat: SatAmount,

    bolt11_invoice: String,
    onchain_address: String,
    required_onchain_block_confirmations: u8,

    minimum_fee_for_0conf: OnchainFeeRate,
    on_chain_payments: Vec<OnchainPayment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2GetVersionsResponse {
    versions: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2GetInfoRequest {
    version: i64,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsp2GetInfoResponse {
    opening_fee_params_menu: Vec<OpeningFeeParamsMenuItem>,
    min_payment_size_msat: String,
    max_payment_size_msat: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningFeeParamsMenuItem {
    min_fee_msat: MsatAmount,
    proportional: u64,
    _valid_until: Datetime,
    min_lifetime: u64,
    max_client_to_self_delay: u64,
    promise: Promise,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2BuyRequest {
    version: String,
    opening_fee_params: OpeningFeeParamsMenuItem,
    payment_size_msat: MsatAmount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2BuyResponse {
    jit_channel_scid : String,
    lsp_cltv_expiry_delta : u64,
    #[serde(default)]
    client_trusts_lsp : bool
}

#[cfg(test)]
mod test {

    use crate::lsps::json_rpc::generate_random_rpc_id;

    use super::*;
    use serde_json::{from_str, to_string, Value};

    #[test]
    fn serialize_request_with_no_params() {
        let method = LSPS0_LIST_PROTOCOLS;
        let json_rpc_id = generate_random_rpc_id();
        let rpc_request = method.create_request_no_params(json_rpc_id);
        let json_str = to_string(&rpc_request).unwrap();

        // Test that params is an empty dict
        //
        // LSPS-0 spec demands that a parameter-by-name scheme is always followed
        let v: Value = from_str(&json_str).unwrap();
        assert_eq!(v.get("jsonrpc").unwrap(), "2.0");
        assert_eq!(
            v.get("method").unwrap().as_str().unwrap(),
            "lsps0.list_protocols"
        );
        assert!(v.get("params").unwrap().as_object().unwrap().is_empty())
    }

    #[test]
    fn serialize_protocol_list() {
        let protocols = ProtocolList {
            protocols: vec![1, 3],
        };

        let json_str = serde_json::to_string(&protocols).unwrap();
        assert_eq!(json_str, "{\"protocols\":[1,3]}")
    }

    #[test]
    fn parsing_error_when_opening_fee_menu_has_extra_fields() {
        // LSPS2 mentions
        // Clients MUST fail and abort the flow if a opening_fee_params object has unrecognized fields.
        //
        // If a new field is added the version number should be incremented
        let fee_menu_item = serde_json::json!(
            {
                "min_fee_msat": "546000",
                "proportional": 1200,
                "valid_until": "2023-02-23T08:47:30.511Z",
                "min_lifetime": 1008,
                "max_client_to_self_delay": 2016,
                "promise": "abcdefghijklmnopqrstuvwxyz",
                "extra_field" : "This shouldn't be their"
            }
        );

        let parsed_opening_fee_menu_item: Result<OpeningFeeParamsMenuItem, _> =
            serde_json::from_value(fee_menu_item);
        assert!(
            parsed_opening_fee_menu_item.is_err_and(|x| format!("{}", x).contains("extra_field"))
        )
    }

    #[test]
    fn parsing_amount_sats() {
        // Pick a number which exceeds 2^32 to ensure internal representation exceeds 32 bits
        let json_str_number = "\"10000000001\"";

        let int_number: u64 = 10000000001;

        let x = serde_json::from_str::<SatAmount>(json_str_number).unwrap();
        assert_eq!(x.0, int_number);
    }

    #[test]
    fn serializing_amount_sats() {
        // Pick a number which exceeds 2^32 to ensure internal representation exceeds 32 bits
        // The json_str includes the " to indicate it is a string
        let json_str_number = "\"10000000001\"";
        let int_number: u64 = 10000000001;

        let sat_amount = SatAmount(int_number);

        let json_str = serde_json::to_string::<SatAmount>(&sat_amount).unwrap();
        assert_eq!(json_str, json_str_number);
    }

    #[test]
    fn parse_and_serialize_datetime() {
        let datetime_str = "\"2023-01-01T23:59:59.999Z\"";

        let dt = serde_json::from_str::<Datetime>(&datetime_str).unwrap();

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

        let result = serde_json::from_str::<Datetime>(&datetime_str);
        result.expect_err("datetime_str should not be parsed if it doesn't follow spec");
    }

    #[test]
    fn parse_valid_promise() {
        let promise_json = "\"abcdefghijklmnopqrstuvwxyz\"";
        let promise = serde_json::from_str::<Promise>(promise_json).expect("Can parse promise");
        assert_eq!(promise.promise, "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn parse_too_long_promise_fails() {
        // Each a char correspond to 1 byte
        let a_513_chars = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";
        let a_512_chars = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";

        serde_json::from_str::<Promise>(a_512_chars)
            .expect("Should fail because 512 bytes is the max length");
        serde_json::from_str::<Promise>(a_513_chars).expect_err("Should fail to parse promise ");
    }

    #[test]
    fn client_trust_lsp_defaults_to_false() {
        let data = serde_json::json!({
            "jit_channel_scid" : "#scid#",
            "lsp_cltv_expiry_delta" : 144
        });

        let buy_response = serde_json::from_value::<Lsps2BuyResponse>(data).expect("The response can be parsed");

        assert!(! buy_response.client_trusts_lsp, "If the field is absent it assumed the client should not trust the LSP")

    }
}
