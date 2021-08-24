pub use libhsmd_sys::Hsmd;

extern crate anyhow;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub mod pb;
pub mod signer;

pub enum Network {
    BITCOIN,
    TESTNET,
    REGTEST,
}

impl Into<&'static str> for Network {
    fn into(self: Network) -> &'static str {
        match self {
            Network::BITCOIN => "bitcoin",
            Network::TESTNET => "testnet",
            Network::REGTEST => "regtest",
        }
    }
}

pub mod tls {
    use tonic::transport::{Certificate, ClientTlsConfig, Identity};
    lazy_static! {
        static ref CA: Certificate = Certificate::from_pem(include_str!("../../../tls/ca.pem"));
        static ref NOBODY: Identity = Identity::from_pem(
            include_str!("../../../tls/users-nobody.pem"),
            include_str!("../../../tls/users-nobody-key.pem")
        );
        pub static ref NOBODY_CONFIG: ClientTlsConfig = ClientTlsConfig::new()
            .domain_name("localhost")
            .ca_certificate(Certificate::from_pem(include_str!("../../../tls/ca.pem")))
            .identity(Identity::from_pem(
                include_str!("../../../tls/users-nobody.pem"),
                include_str!("../../../tls/users-nobody-key.pem")
            ));
    }
}
