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
