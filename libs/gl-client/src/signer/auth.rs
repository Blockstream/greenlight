//! Utilities used to authorize a signature request based on pending RPCs
use crate::signer::model::Request;
use crate::Error;
use lightning_signer::invoice::Invoice;
use std::str::FromStr;
use vls_protocol_signer::approver::Approval;

fn normalize_invoice(value: &str) -> String {
    let normalized = value.to_lowercase();
    normalized
        .strip_prefix("lightning:")
        .unwrap_or(&normalized)
        .to_owned()
}

pub(super) fn parse_invoice(value: &str) -> anyhow::Result<Invoice> {
    Invoice::from_str(&normalize_invoice(value))
        .map_err(|e| anyhow::anyhow!("invalid payment invoice: {e}"))
}

pub(super) fn same_invoice(wire: &[u8], context: &str) -> bool {
    std::str::from_utf8(wire)
        .is_ok_and(|wire| normalize_invoice(wire) == normalize_invoice(context))
}

pub trait Authorizer {
    fn authorize(&self, requests: &Vec<Request>) -> Result<Vec<Approval>, Error>;
}

pub struct GreenlightAuthorizer {}

impl Authorizer for GreenlightAuthorizer {
    fn authorize(&self, requests: &Vec<Request>) -> Result<Vec<Approval>, Error> {
        let mut approvals = Vec::new();
        for request in requests.iter() {
            let invoice = match request {
                Request::Pay(req) => Some(req.bolt11.as_str()),
                Request::Xpay(req) => Some(req.invstring.as_str()),
                Request::SendPay(req) => req.bolt11.as_deref(),
                _ => None,
            };
            if let Some(invoice) = invoice {
                match parse_invoice(invoice) {
                    Ok(invoice) => approvals.push(Approval::Invoice(invoice)),
                    Err(e) => {
                        // One invalid pending command must not reject
                        // unrelated signing requests in the same context.
                        log::warn!("Skipping invalid invoice approval: {e:#}");
                    }
                }
            }
        }
        Ok(approvals)
    }
}

#[cfg(test)]
mod test {
    use crate::signer::auth::Authorizer;
    use crate::signer::auth::GreenlightAuthorizer;
    use crate::signer::model::Request;
    use cln_grpc::pb::PayRequest;

    #[test]
    fn test_prefix_stripping() {
        let invs = vec![
	    "lightning:lnbc80u1p4zzlrqdqqpp5uncavtdcq8k9rw0ef0q0d6a7vr0cktxrgu5azs0glssvtx6l64escqzp2sp55d0jdhcgffylnh04kawkn3926quhlegsa0zf9renpvkpq9ptc6dq9qyysgqxqyz5vqnp4qwxa7dsnsmy72v39u4qj9x2ly7k57ye3ytuvjpmdc4fwnwkn5yqgzrzjqv22wafr68wtchd4vzq7mj7zf2uzpv67xsaxcemfzak7wp7p0r29wr8aasqqtrcqqvqqqqqqqqqqhwqqfq6e5scsajygx83z5kqaswk0ulzkuqgp22c5ltc9jrw4shrtszw4fzcgx38fmr6e06r8s3rl5z4ck0pl3gyzmfglv3efth304330ccu3sqc9uz8q",
	    "lnbc80u1p4zzlrqdqqpp5uncavtdcq8k9rw0ef0q0d6a7vr0cktxrgu5azs0glssvtx6l64escqzp2sp55d0jdhcgffylnh04kawkn3926quhlegsa0zf9renpvkpq9ptc6dq9qyysgqxqyz5vqnp4qwxa7dsnsmy72v39u4qj9x2ly7k57ye3ytuvjpmdc4fwnwkn5yqgzrzjqv22wafr68wtchd4vzq7mj7zf2uzpv67xsaxcemfzak7wp7p0r29wr8aasqqtrcqqvqqqqqqqqqqhwqqfq6e5scsajygx83z5kqaswk0ulzkuqgp22c5ltc9jrw4shrtszw4fzcgx38fmr6e06r8s3rl5z4ck0pl3gyzmfglv3efth304330ccu3sqc9uz8q",
	    "LIGHTNING:LNBC80U1P4ZZLRQDQQPP5UNCAVTDCQ8K9RW0EF0Q0D6A7VR0CKTXRGU5AZS0GLSSVTX6L64ESCQZP2SP55D0JDHCGFFYLNH04KAWKN3926QUHLEGSA0ZF9RENPVKPQ9PTC6DQ9QYYSGQXQYZ5VQNP4QWXA7DSNSMY72V39U4QJ9X2LY7K57YE3YTUVJPMDC4FWNWKN5YQGZRZJQV22WAFR68WTCHD4VZQ7MJ7ZF2UZPV67XSAXCEMFZAK7WP7P0R29WR8AASQQTRCQQVQQQQQQQQQQHWQQFQ6E5SCSAJYGX83Z5KQASWK0ULZKUQGP22C5LTC9JRW4SHRTSZW4FZCGX38FMR6E06R8S3RL5Z4CK0PL3GYZMFGLV3EFTH304330CCU3SQC9UZ8Q",
	];

        let auth = GreenlightAuthorizer {};
        for inv in invs {
            let reqs = vec![Request::Pay(PayRequest {
                bolt11: inv.to_string(),
                ..Default::default()
            })];

            let approvals = dbg!(auth.authorize(&reqs).unwrap());
            assert_eq!(approvals.len(), 1);
        }
    }
}
