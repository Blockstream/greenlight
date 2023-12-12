use tonic::transport::Channel;

use crate::{
    credentials::{RuneProvider, TlsConfigProvider},
    pb::scheduler::{pairing_client::PairingClient, GetPairingDataRequest, GetPairingDataResponse},
    tls::TlsConfig,
};

type Result<T, E = super::Error> = core::result::Result<T, E>;

pub struct Connected(PairingClient<Channel>);
pub struct Unconnected();

pub struct Client<T> {
    inner: T,
    tls: TlsConfig,
    uri: String,
}

impl Client<Unconnected> {
    pub fn new<C>(creds: C) -> Result<Client<Unconnected>>
    where
        C: TlsConfigProvider + RuneProvider,
    {
        let tls = creds.tls_config();
        Ok(Self {
            inner: Unconnected(),
            tls,
            uri: crate::utils::scheduler_uri(),
        })
    }

    pub fn with_uri(mut self, uri: String) -> Client<Unconnected> {
        self.uri = uri;
        self
    }

    pub async fn connect(self) -> Result<Client<Connected>> {
        let channel = tonic::transport::Endpoint::from_shared(self.uri.clone())?
            .tls_config(self.tls.inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let inner = PairingClient::new(channel);

        Ok(Client {
            inner: Connected(inner),
            tls: self.tls,
            uri: self.uri,
        })
    }
}

impl Client<Connected> {
    pub async fn get_pairing_data(&self, session_id: &str) -> Result<GetPairingDataResponse> {
        Ok(self
            .inner
            .0
            .clone()
            .get_pairing_data(GetPairingDataRequest {
                session_id: session_id.to_string(),
            })
            .await?
            .into_inner())
    }
}
