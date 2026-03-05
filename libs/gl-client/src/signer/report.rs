//! Signer reporting facility to debug issues
//!
//! The resolver and policies implemented in the signer may produce
//! false negatives, i.e., they may reject an otherwise valid request
//! based on a missing approval or failing to match up the request
//! with the signed context requests in the resolver.
//!
//! Since issues involving these are hard to debug, given that they
//! run on user devices, we'd like to report any failure to the
//! servers where they are logged and used to fine-tune policies and
//! the resolver. The information in these reports is already known by
//! the server and we are attaching most of it just for easier
//! collation by capturing the full context.

use crate::pb;

pub const STATE_SIGNATURE_OVERRIDE_ENABLED_PREFIX: &str = "STATE_SIGNATURE_OVERRIDE_ENABLED";
pub const STATE_SIGNATURE_OVERRIDE_USED_PREFIX: &str = "STATE_SIGNATURE_OVERRIDE_USED";
const KEY_LOG_LIMIT: usize = 8;

pub struct Reporter {}

impl Reporter {
    pub async fn report(r: pb::scheduler::SignerRejection) {
        log::warn!("Delivering report {:?}", r);
        let tls = crate::tls::TlsConfig::new();
        let uri = crate::utils::scheduler_uri();
        let channel = tonic::transport::Endpoint::from_shared(uri)
            .expect("could not configure client")
            .tls_config(tls.inner.clone())
            .expect("error configuring client with tls config")
            .connect_lazy();

        let mut client = pb::scheduler::debug_client::DebugClient::new(channel);
        match client.report_signer_rejection(r).await {
            Ok(_) => log::info!("rejection reported"),
            Err(e) => log::error!("could not report rejection: {}", e),
        }
    }
}

fn summarize_keys(keys: &[String]) -> String {
    if keys.is_empty() {
        return "-".to_string();
    }

    let mut listed = keys
        .iter()
        .take(KEY_LOG_LIMIT)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");
    let hidden = keys.len().saturating_sub(KEY_LOG_LIMIT);
    if hidden > 0 {
        listed.push_str(&format!(",+{}more", hidden));
    }
    listed
}

fn format_note(note: Option<&str>) -> String {
    note.map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "null".to_string())
}

pub fn build_state_signature_override_enabled_message(
    mode: &str,
    node_id: &[u8],
    note: Option<&str>,
) -> String {
    format!(
        "{} mode={} node_id={} note={}",
        STATE_SIGNATURE_OVERRIDE_ENABLED_PREFIX,
        mode,
        hex::encode(node_id),
        format_note(note),
    )
}

pub fn build_state_signature_override_used_message(
    mode: &str,
    request_id: u64,
    missing_keys: &[String],
    invalid_keys: &[String],
    note: Option<&str>,
) -> String {
    format!(
        "{} mode={} request_id={} missing_count={} invalid_count={} missing_keys={} invalid_keys={} note={}",
        STATE_SIGNATURE_OVERRIDE_USED_PREFIX,
        mode,
        request_id,
        missing_keys.len(),
        invalid_keys.len(),
        summarize_keys(missing_keys),
        summarize_keys(invalid_keys),
        format_note(note),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_state_signature_override_enabled_message, build_state_signature_override_used_message,
        STATE_SIGNATURE_OVERRIDE_ENABLED_PREFIX, STATE_SIGNATURE_OVERRIDE_USED_PREFIX,
    };

    #[test]
    fn override_enabled_message_contains_required_fields() {
        let msg = build_state_signature_override_enabled_message(
            "soft",
            &[0x02, 0xab, 0xcd],
            Some("operator assisted"),
        );
        assert!(msg.starts_with(STATE_SIGNATURE_OVERRIDE_ENABLED_PREFIX));
        assert!(msg.contains("mode=soft"));
        assert!(msg.contains("node_id=02abcd"));
        assert!(msg.contains("note=\"operator assisted\""));
    }

    #[test]
    fn override_used_message_contains_required_fields() {
        let missing = vec!["nodes/a".to_string()];
        let invalid = vec!["nodes/b".to_string(), "nodes/c".to_string()];
        let msg = build_state_signature_override_used_message(
            "hard",
            7,
            &missing,
            &invalid,
            Some("manual fix"),
        );
        assert!(msg.starts_with(STATE_SIGNATURE_OVERRIDE_USED_PREFIX));
        assert!(msg.contains("mode=hard"));
        assert!(msg.contains("request_id=7"));
        assert!(msg.contains("missing_count=1"));
        assert!(msg.contains("invalid_count=2"));
        assert!(msg.contains("missing_keys=nodes/a"));
        assert!(msg.contains("invalid_keys=nodes/b,nodes/c"));
        assert!(msg.contains("note=\"manual fix\""));
    }
}
