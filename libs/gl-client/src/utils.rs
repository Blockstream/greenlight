use crate::tls::TlsConfig;
use anyhow::{anyhow, Result};

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

    // Must have at least 3 components: "" / "users" / "node_id"
    if split_subject_common_name.len() < 3 {
        return Err(anyhow!(
            "Could not parse subject common name: {}",
            subject_common_name
        ));
    } else if split_subject_common_name[1] != "users" {
        return Err(anyhow!("Not a users certificate: {}", subject_common_name));
    }

    hex::decode(split_subject_common_name[2]).map_err(|e| {
        anyhow!(
            "Unable to decode node_id ({}): {}",
            split_subject_common_name[2],
            e
        )
    })
}
