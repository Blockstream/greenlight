use lightning_signer::prelude::SendSync;
use vls_protocol_signer::approver::Approve;

// An approver that will collect any request it gets and files a
// report that may be relayed to developers to debug policies. It
// defers actual decisions to an `inner` approver, and provides access
// to the captured reports. If this approver is wrapped in a real
// approver, that outer approver will process the requests, and not
// forward to this. Hence only prospective failures are collected.
pub struct ReportingApprover<A: Approve> {
    inner: A,
}

impl<A: Approve> ReportingApprover<A> {
    pub fn new(delegate: A) -> Self {
        ReportingApprover { inner: delegate }
    }
}

impl<A: Approve> Approve for ReportingApprover<A> {
    fn approve_invoice(&self, inv: &lightning_signer::invoice::Invoice) -> bool {
	log::warn!("unapproved invoice: {:?}", inv);
        self.inner.approve_invoice(inv)
    }
    fn approve_keysend(
        &self,
        hash: crate::lightning::ln::PaymentHash,
        amount_msat: u64,
    ) -> bool {
        log::warn!("unapproved keysend {:?} {:?}", hash, amount_msat);
        self.inner.approve_keysend(hash, amount_msat)
    }
    fn approve_onchain(
        &self,
        tx: &lightning_signer::bitcoin::Transaction,
        values_sat: &[lightning_signer::bitcoin::TxOut],
        unknown_indices: &[usize],
    ) -> bool {
        log::warn!(
            "unapproved onchain {:?} {:?} {:?}",
            tx,
            values_sat,
            unknown_indices
        );
        self.inner.approve_onchain(tx, values_sat, unknown_indices)
    }
}
impl<A: Approve> SendSync for ReportingApprover<A> {}
