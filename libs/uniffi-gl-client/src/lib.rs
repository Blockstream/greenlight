uniffi_macros::include_scaffolding!("gl_client");

use std::sync::Arc;

pub use gl_client::tls::TlsConfig;

fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn default_tls_config() -> Arc<TlsConfig> {
    Arc::new(TlsConfig::new().unwrap())
}

fn print_tls(tls_config: &TlsConfig) -> String {
    format!("{:?}", tls_config.ca)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = super::add(2, 2);
        assert_eq!(result, 4);
    }
}
