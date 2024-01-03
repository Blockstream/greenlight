//! Resolver utilities to match incoming requests against the request
//! context and find a justifications.

use crate::signer::{model::Request, Error};
use vls_protocol::msgs::Message;
pub struct Resolver {}

impl Resolver {
    /// Attempt to find a resolution for a given request. We default
    /// to failing, and allowlist individual matches between pending
    /// context requests and the signer request being resolved. Where
    /// possible we also verify the contents of the request against
    /// the contents of the context request. TODOs in here may
    /// indicate ways to strengthen the verification.
    pub fn try_resolve(req: &Message, reqctx: &Vec<Request>) -> Result<(), Error> {
        log::trace!("Resolving {:?}", req);
        // Some requests do not need a justification. For example we
        // reconnect automatically, so there may not even be a context
        // request pending which would skip the entire stack below, so
        // we do an early pass:
        let accept = match req {
            // Commands that simply have no context to check against
            Message::GetHeartbeat(_) => true,
            Message::Ecdh(_) => true,
            Message::Ping(_) => true,
            Message::Pong(_) => true,
            Message::SignChannelAnnouncement(_) => true,
            Message::SignChannelUpdate(_) => true,
            Message::SignNodeAnnouncement(_) => true,
            Message::CheckPubKey(_) => true,
            // Duplicate verification with VLS, we defer to VLS
            Message::GetChannelBasepoints(_) => true,
            Message::ValidateCommitmentTx(_) => true,
            Message::SignWithdrawal(_) => true,
            Message::SetupChannel(_) => true,
            Message::GetPerCommitmentPoint(_) => true,
            Message::ValidateRevocation(_) => true,
            Message::NewChannel(_) => true,
            Message::SignCommitmentTx(_) => true,
            Message::SignGossipMessage(_) => true,
            Message::SignMutualCloseTx(_) => true,
            Message::SignMutualCloseTx2(_) => true,
            Message::SignRemoteCommitmentTx(_) => true,
            Message::SignRemoteCommitmentTx2(_) => true,
            Message::SignRemoteHtlcTx(_) => true,
            // Resolution of an existing HTLC, we should never not try
            // to grab funds if we can.
            Message::SignPenaltyToUs(_) => true,
            Message::SignAnyPenaltyToUs(_) => true,
            Message::SignAnyDelayedPaymentToUs(_) => true,
            Message::SignAnyLocalHtlcTx(_) => true,
            Message::SignAnyRemoteHtlcToUs(_) => true,
            // Default to rejecting, punting the decision to the next
            // step.
            _ => false,
        };

        // If we found a resolution, then there is no point in trying
        // to match up further.
        if accept {
            log::trace!(
                "Request {:?} resolved with no context request required",
                req
            );
            return Ok(());
        }

        for cr in reqctx {
            let accept = match (req, cr) {
                (Message::SignMessage(m1), Request::SignMessage(m2)) => {
                    m1.message.0.clone() == m2.message.as_bytes()
                }
                (Message::NewChannel(m1), Request::FundChannel(m2)) => {
                    // Different node_id? Reject!
                    m1.node_id.0 == m2.id.as_slice()
                    // TODO: Add `close_to` to allowlist for the close
                    // later on
                }
                (Message::NewChannel(m1), Request::GlFundChannel(m2)) => {
                    // Different node_id? Reject!
                    m1.node_id.0 == m2.node_id.as_slice()
                }
                (Message::SignInvoice(_l), Request::GlCreateInvoice(_r)) => true,
                (Message::SignInvoice(_l), Request::Invoice(_r)) => {
                    // TODO: This could be strengthened by parsing the
                    // invoice from `l.u5bytes` and verify the
                    // description, amount and (maybe) payment_hash
                    true
                }
                (Message::PreapproveInvoice(l), Request::Pay(r)) => {
                    l.invstring.0 == r.bolt11.as_bytes()
                }
                (Message::PreapproveInvoice(l), Request::PreApproveInvoice(r)) => {
                    // Manually calling preapproveinvoice should
                    // always be allowed. The bolt11 string have to
                    // match.
                    l.invstring.0 == r.bolt11().as_bytes()
                }
                (_, _) => false,
            };

            // Did we find a resolution? If yes we can stop here.
            if accept {
                log::trace!("Request {:?} approved with context request {:?}", req, cr);
                return Ok(());
            }
        }

        let ser = req.inner().as_vec();
        Err(Error::Resolver(ser, reqctx.to_vec()))
    }
}
