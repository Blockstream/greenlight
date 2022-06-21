use bitcoin::Network;
use log::debug;
use neon::prelude::*;
use tokio::sync::mpsc;
use log::warn;
#[derive(Clone)]
pub struct Signer {
    pub(crate) inner: gl_client::signer::Signer,
}

#[derive(Clone)]
pub struct SignerHandle {
    pub(crate) signal: mpsc::Sender<()>,
}

impl Signer {
    pub(crate) fn new(mut cx: FunctionContext) -> JsResult<JsBox<Signer>> {
        let buf = cx.argument::<JsBuffer>(0)?;
        let secret: Vec<u8> = cx.borrow(&buf, |data| data.as_slice().to_vec());
        let network: String = cx.argument::<JsString>(1)?.value(&mut cx);
        let tls = (&**cx.argument::<JsBox<crate::TlsConfig>>(2)?).clone();

        let network: Network = network
            .parse()
            .unwrap_or_else(|_| panic!("Unknown / unsupported network {}", network));

        let inner = match gl_client::signer::Signer::new(secret, network, tls.inner) {
            Ok(v) => v,
            Err(e) => {
                panic!("Error initializing Signer: {}", e)
            }
        };
        Ok(cx.boxed(Signer { inner }))
    }

    pub(crate) fn run_in_thread(mut cx: FunctionContext) -> JsResult<JsBox<SignerHandle>> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        let inner = this.inner.clone();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, rx) = mpsc::channel(1);

        std::thread::spawn(move || {
            runtime.block_on(async move {
                debug!("Running signer");
                inner.run_forever(rx).await
            })
        });
        Ok(cx.boxed(SignerHandle { signal: tx }))
    }

    pub(crate) fn run_forever(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        let (_tx, rx) = mpsc::channel(1);
        let _ = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(this.inner.run_forever(rx));
        Ok(cx.undefined())
    }

    pub(crate) fn node_id(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        let node_id = this.inner.node_id();
        let buf = JsBuffer::new(&mut cx, 33)?;
        cx.borrow(&buf, |buf| buf.as_mut_slice().copy_from_slice(&node_id));
        Ok(buf)
    }
    pub(crate) fn version(mut cx: FunctionContext) -> JsResult<JsString> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        Ok(JsString::new(&mut cx, this.inner.version()))
    }
}

impl SignerHandle {
    pub(crate) fn shutdown(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.argument::<JsBox<SignerHandle>>(0)?;
        if let Err(e) = this.signal.try_send(()) {
	    warn!("Failed to send shutdown signal, signer may already be stopped: {e}");
	}
        Ok(cx.undefined())
    }
}

impl Finalize for Signer {}
impl Finalize for SignerHandle {}
