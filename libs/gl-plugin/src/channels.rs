//! A single rule that decides whether a channel is *available*.
//!
//! A channel is available when it can be used to send or receive, can be
//! advertised as a routehint, and is worth awaiting (re-)establishment
//! for. Unavailable channels must be skipped everywhere: awaiting them
//! stalls payments until the wait deadline expires, and advertising them
//! in invoices routes payments into a black hole.
//!
//! The rule is:
//!
//! ```text
//! available := state == CHANNELD_NORMAL
//!              && (max_to_us_msat > 0 || short_channel_id.is_some())
//! ```
//!
//! `max_to_us_msat` is a monotonically increasing high-water mark of the
//! balance we have ever held on this channel. For JIT channels (zeroconf
//! channels opened by an LSP server to carry an incoming payment, which
//! may not have a `short_channel_id` yet) a non-zero gauge is the signal
//! that the incoming JIT payment that caused the channel to be opened
//! was claimed: the LSP server was paid and has no reason to abandon the
//! channel, so awaiting it can succeed. A channel that has never held
//! funds (`max_to_us_msat == 0`) *and* was never confirmed (no
//! `short_channel_id`) is a dangling channel open, most likely an
//! abandoned JIT channel whose LSP server may have dropped the channel;
//! it will never come back, so we neither await it nor advertise it.
//!
//! The current balance (`spendable_msat`) is deliberately secondary: a
//! drained but confirmed channel remains available (it may be refilled
//! or come back online), while a never-funded zeroconf channel is not,
//! even if it would momentarily show spendable capacity.

use std::collections::HashSet;
use std::hash::Hash;

/// Decide whether a channel is available for payments, awaiting and
/// routehints.
///
/// The arguments are deliberately primitive so the rule can be applied
/// to whatever channel representation a call site has (RPC responses,
/// cached structs, …) without depending on a particular cln-rpc version.
pub fn is_channel_available(
    state_is_normal: bool,
    has_scid: bool,
    max_to_us_msat_msat: u64,
) -> bool {
    state_is_normal && (max_to_us_msat_msat > 0 || has_scid)
}

/// The peers worth reconnecting to: those with at least one available
/// channel.
///
/// Reconnecting is only useful to bring channels back. A peer with no
/// channel at all, or with only unavailable ones (e.g. an abandoned JIT
/// channel the LSP server may have forgotten), is not worth dialing, and
/// an unreachable one can hold up the caller for minutes per `connect`.
///
/// Each item is `(peer_id, state_is_normal, has_scid, max_to_us_msat)`,
/// see [`is_channel_available`].
pub fn peers_with_available_channels<K, I>(channels: I) -> HashSet<K>
where
    K: Eq + Hash,
    I: IntoIterator<Item = (K, bool, bool, u64)>,
{
    channels
        .into_iter()
        .filter(|(_, normal, scid, max_to_us)| is_channel_available(*normal, *scid, *max_to_us))
        .map(|(id, ..)| id)
        .collect()
}

/// Ask `lightningd` for the peers worth reconnecting to, as lowercase
/// hex node ids. See [`peers_with_available_channels`].
pub async fn reconnect_candidates(
    rpc: &mut cln_rpc::ClnRpc,
) -> Result<HashSet<String>, cln_rpc::RpcError> {
    use cln_rpc::primitives::ChannelState;

    let res = rpc
        .call_typed(&cln_rpc::model::requests::ListpeerchannelsRequest { id: None })
        .await?;

    Ok(peers_with_available_channels(res.channels.into_iter().map(
        |c| {
            (
                c.peer_id.to_string(),
                c.state == ChannelState::CHANNELD_NORMAL,
                c.short_channel_id.is_some(),
                c.max_to_us_msat.map(|a| a.msat()).unwrap_or(0),
            )
        },
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability_truth_table() {
        // Confirmed and normal: always available, even never funded
        // (e.g. an incoming-only channel) or currently drained.
        assert!(is_channel_available(true, true, 0));
        assert!(is_channel_available(true, true, 1));

        // Zeroconf (no scid) and the JIT payment was claimed: available,
        // the LSP server has no reason to have dropped the channel.
        assert!(is_channel_available(true, false, 1));

        // Never funded and never confirmed: a dangling channel open,
        // most likely an abandoned JIT channel, never await/hint it.
        assert!(!is_channel_available(true, false, 0));

        // Whatever the gauge says, a channel that is not
        // CHANNELD_NORMAL can not be used or awaited right now.
        assert!(!is_channel_available(false, true, 1));
        assert!(!is_channel_available(false, false, 1));
        assert!(!is_channel_available(false, true, 0));
        assert!(!is_channel_available(false, false, 0));
    }

    #[test]
    fn test_reconnect_needs_one_available_channel() {
        let peers = peers_with_available_channels(vec![
            // Confirmed normal channel: reconnect.
            ("good", true, true, 0),
            // A stalled JIT channel next to a good one still reconnects.
            ("mixed", true, false, 0),
            ("mixed", true, true, 1),
            // Only an abandoned JIT channel: skip.
            ("stalled-jit", true, false, 0),
            // Only a closing channel: skip.
            ("closing", false, true, 1),
        ]);
        let mut peers: Vec<_> = peers.into_iter().collect();
        peers.sort();
        assert_eq!(peers, vec!["good", "mixed"]);
    }
}
