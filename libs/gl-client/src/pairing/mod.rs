use crate::{credentials, pb::scheduler::PairDeviceResponse};
use anyhow::{ensure, Context};
use runeauth::{Condition, Restriction};
use thiserror::Error;

pub mod attestation_device;
pub mod new_device;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
    #[error(transparent)]
    X509Error(#[from] rcgen::Error),
    #[error("could not build client: {0}")]
    BuildClientError(String),
    #[error(transparent)]
    GrpcError(#[from] tonic::Status),
    #[error(transparent)]
    CredentialsError(#[from] credentials::Error),
    #[error(transparent)]
    RuneError(#[from] runeauth::RuneError),
    #[error("could not approve pairing: {0}")]
    ApprovePairingError(String),
    #[error("could not verify pairing data: {0}")]
    VerifyPairingDataError(String),
}

#[derive(Debug)]
pub enum PairingSessionData {
    PairingResponse(PairDeviceResponse),
    PairingQr(String),
    PairingError(tonic::Status),
}

pub(crate) fn parse_restrictions(
    device_id: &str,
    restrictions: &str,
) -> anyhow::Result<Vec<Restriction>> {
    // A device ID is the lowercase hex encoding of an uncompressed P-256 key.
    ensure!(
        device_id.len() == 130
            && device_id.starts_with("04")
            && device_id
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')),
        "invalid pairing device ID"
    );
    // runeauth 0.1.4 uses character counts to calculate byte offsets.
    ensure!(
        restrictions.is_ascii(),
        "pairing restrictions must be ASCII"
    );

    let (binding, mut remaining) =
        Restriction::decode(restrictions, false).context("invalid pairing restriction syntax")?;
    // The binding goes first to delimit the restrictions in the signed approval.
    ensure!(
        matches!(binding.alternatives.as_slice(), [alternative]
            if alternative.get_field() == "pubkey"
                && alternative.get_condition() == Condition::Equal
                && alternative.get_value() == device_id),
        "pairing restrictions must start with a standalone pubkey equality matching the device ID"
    );

    let mut parsed = vec![binding];
    while !remaining.is_empty() {
        let (restriction, rest) =
            Restriction::decode(remaining, false).context("invalid pairing restriction syntax")?;
        match restriction.alternatives.as_slice() {
            [alternative] if alternative.get_field() == "time" => {
                let value = alternative.get_value();
                ensure!(
                    alternative.get_condition() == Condition::IntLT
                        && value.bytes().all(|byte| byte.is_ascii_digit())
                        && value.parse::<i64>().is_ok(),
                    "pairing expiry must be an integer timestamp upper bound"
                );
            }
            alternatives => ensure!(
                alternatives.iter().all(|alternative| {
                    let value = alternative.get_value();
                    alternative.get_field() == "method"
                        && matches!(
                            alternative.get_condition(),
                            Condition::Equal | Condition::BeginsWith
                        )
                        && !value.is_empty()
                        && value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                }),
                "pairing method clauses must contain only method names or nonempty method prefixes"
            ),
        }
        parsed.push(restriction);
        remaining = rest;
    }

    // Reject syntax the decoder discards, such as a trailing separator.
    let canonical = parsed
        .iter()
        .map(Restriction::encode)
        .collect::<Vec<_>>()
        .join("&");
    ensure!(
        canonical == restrictions,
        "pairing restrictions must use canonical rune syntax"
    );
    Ok(parsed)
}

fn into_approve_pairing_error<T: ToString>(v: T) -> Error {
    Error::ApprovePairingError(v.to_string())
}

fn into_verify_pairing_data_error<T: ToString>(v: T) -> Error {
    Error::VerifyPairingDataError(v.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_restrictions;
    use crate::tls::generate_ecdsa_key_pair;

    #[test]
    fn pairing_policy_validation() {
        let device_id = hex::encode(generate_ecdsa_key_pair().public_key_raw());
        let binding = format!("pubkey={device_id}");

        for policy in [
            binding.clone(),
            format!("{binding}&method^list|method=getinfo&time<2000000000"),
        ] {
            assert!(parse_restrictions(&device_id, &policy).is_ok(), "{policy}");
        }
        let other_device_id = hex::encode(generate_ecdsa_key_pair().public_key_raw());
        assert!(parse_restrictions(&other_device_id, &binding).is_err());

        for policy in [
            String::new(),
            format!("method=getinfo&{binding}"),
            format!("{binding}&{binding}"),
            format!("{binding}&"),
            format!("{binding}&method=getinfo|"),
            format!("{binding}&method^"),
            format!("{binding}&method=gétinfo&method=getinfo"),
            format!("{binding}&time<tomorrow"),
            format!("{binding}&unknown=getinfo"),
        ] {
            assert!(parse_restrictions(&device_id, &policy).is_err(), "{policy}");
        }
    }
}
