use serde::de::Error as DeError;
use serde::{Deserialize, Serialize};

use crate::lsps::error::map_json_rpc_error_code_to_str;
use crate::lsps::json_rpc::MapErrorCode;
use crate::lsps::lsps0::common_schemas::{IsoDatetime, MsatAmount, ShortChannelId};

const MAX_PROMISE_LEN_BYTES: usize = 512;
#[derive(Debug)]
struct Promise {
    promise: String,
}

impl Promise {
    fn new(promise: String) -> Result<Self, String> {
        if promise.len() <= MAX_PROMISE_LEN_BYTES {
            Ok(Promise { promise })
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
        serializer.serialize_str(&self.promise)
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
        Promise::new(str_repr.clone())
            .map_err(|_| D::Error::custom("promise exceeds max length"))
    }
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
pub struct Lsps2GetInfoResponse {
    opening_fee_params_menu: Vec<OpeningFeeParamsMenuItem>,
    min_payment_size_msat: String,
    max_payment_size_msat: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2GetInfoError {}

impl MapErrorCode for Lsps2GetInfoError {
    fn get_code_str(code: i64) -> &'static str {
        match code {
            1 => "unsupported_version",
            2 => "unrecognized_or_stale_token",
            _ => map_json_rpc_error_code_to_str(code),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningFeeParamsMenuItem {
    min_fee_msat: MsatAmount,
    proportional: u64,
    valid_until: IsoDatetime,
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
    jit_channel_scid: ShortChannelId,
    lsp_cltv_expiry_delta: u64,
    #[serde(default)]
    client_trusts_lsp: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2BuyError {}

impl MapErrorCode for Lsps2BuyError {
    fn get_code_str(code: i64) -> &'static str {
        match code {
            1 => "unsupported_version",
            2 => "invalid_opening_fee_params",
            3 => "payment_size_too_small",
            4 => "payment_size_too_large",
            _ => map_json_rpc_error_code_to_str(code),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_error_when_opening_fee_menu_has_extra_fields() {
        // LSPS2 mentions
        // Clients MUST fail and abort the flow if a opening_fee_params object has unrecognized fields.
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
    fn parse_valid_promise() {
        let promise_json = "\"abcdefghijklmnopqrstuvwxyz\"";
        let promise = serde_json::from_str::<Promise>(promise_json).expect("Can parse promise");
        assert_eq!(promise.promise, "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn parse_too_long_promise_fails() {
        // Each a char correspond to 1 byte
        // We refuse to parse the promise if it is too long
        // LSPS2 requires us to ignore Promise that are too long
        // so the client cannot be burdened with unneeded storage requirements
        let a_513_chars = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";
        let a_512_chars = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";

        serde_json::from_str::<Promise>(a_512_chars)
            .expect("Should fail because 512 bytes is the max length");
        serde_json::from_str::<Promise>(a_513_chars).expect_err("Should fail to parse promise ");
    }

    #[test]
    fn client_trust_lsp_defaults_to_false() {
        let data = serde_json::json!({
            "jit_channel_scid" : "0x12x12",
            "lsp_cltv_expiry_delta" : 144
        });

        let buy_response =
            serde_json::from_value::<Lsps2BuyResponse>(data).expect("The response can be parsed");

        assert!(
            !buy_response.client_trusts_lsp,
            "If the field is absent it assumed the client should not trust the LSP"
        )
    }
}
