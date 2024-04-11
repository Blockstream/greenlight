use anyhow::{anyhow, Result};
use crate::tls::TlsConfig;

pub fn scheduler_uri() -> String {
    std::env::var("GL_SCHEDULER_GRPC_URI")
        .unwrap_or_else(|_| "https://scheduler.gl.blckstrm.com".to_string())
}

pub fn get_node_id_from_tls_config(tls_config: &TlsConfig) -> Result<Vec<u8>> {
    let subject_common_name = match &tls_config.x509_cert {
        Some(x) => match x.subject_common_name() {
            Some(cn) => cn,
            None => {
                return Err(anyhow!(
                    "Failed to parse the subject common name in the provided x509 certificate"
                ))
            }
        },
        None => {
            return Err(anyhow!(
                "The certificate could not be parsed in the x509 format"
            ))
        }
    };

    let split_subject_common_name = subject_common_name.split("/").collect::<Vec<&str>>();

    assert!(split_subject_common_name[1] == "users");
    Ok(hex::decode(split_subject_common_name[2])
        .expect("Failed to parse the node_id from the TlsConfig to bytes"))
}