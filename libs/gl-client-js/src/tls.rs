use gl_client::tls;
use neon::prelude::*;

#[derive(Clone, Default)]
pub struct TlsConfig {
    pub(crate) inner: tls::TlsConfig,
}

impl Finalize for TlsConfig {}

impl TlsConfig {
    pub(crate) fn new(mut cx: FunctionContext) -> JsResult<JsBox<TlsConfig>> {
        Ok(cx.boxed(Self {
            inner: tls::TlsConfig::default(),
        }))
    }

    pub(crate) fn identity(mut cx: FunctionContext) -> JsResult<JsBox<TlsConfig>> {
        let this = cx.argument::<JsBox<TlsConfig>>(0)?;
        let buf = cx.argument::<JsBuffer>(1)?;
        let cert_pem: Vec<u8> = cx.borrow(&buf, |data| data.as_slice().to_vec());
        let buf = cx.argument::<JsBuffer>(2)?;
        let key_pem: Vec<u8> = cx.borrow(&buf, |data| data.as_slice().to_vec());
        Ok(cx.boxed(Self {
            inner: this.inner.clone().identity(cert_pem, key_pem),
        }))
    }
}
