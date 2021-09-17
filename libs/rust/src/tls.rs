use tonic::transport::{Certificate, ClientTlsConfig, Identity};

lazy_static! {
    pub static ref CA: Certificate = Certificate::from_pem(include_str!("../tls/ca.pem"));
    pub static ref NOBODY: Identity = Identity::from_pem(
        include_str!("../tls/users-nobody.pem"),
        include_str!("../tls/users-nobody-key.pem")
    );
    pub static ref NOBODY_CONFIG: ClientTlsConfig = ClientTlsConfig::new()
        .domain_name("localhost")
        .ca_certificate(Certificate::from_pem(include_str!("../tls/ca.pem")))
        .identity(Identity::from_pem(
            include_str!("../tls/users-nobody.pem"),
            include_str!("../tls/users-nobody-key.pem")
        ));
}
