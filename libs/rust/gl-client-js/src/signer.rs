use gl_client::Network;
use log::debug;
use neon::prelude::*;

#[derive(Clone)]
pub struct Signer {
    pub(crate) inner: gl_client::signer::Signer,
}

impl Signer {
    pub(crate) fn new(mut cx: FunctionContext) -> JsResult<JsBox<Signer>> {
        let buf = cx.argument::<JsBuffer>(0)?;
        let secret: Vec<u8> = cx.borrow(&buf, |data| data.as_slice().to_vec());
        let network: String = cx.argument::<JsString>(1)?.value(&mut cx);
        let tls = (&**cx.argument::<JsBox<crate::TlsConfig>>(2)?).clone();

        let network = match network.as_str() {
            "bitcoin" => Network::BITCOIN,
            "testnet" => Network::TESTNET,
            "regtest" => Network::REGTEST,
            v => panic!("Unknown / unsupported network {}", v),
        };

        let inner = match gl_client::signer::Signer::new(secret, network, tls.inner) {
            Ok(v) => v,
            Err(e) => {
                panic!("Error initializing Signer: {}", e)
            }
        };
        Ok(cx.boxed(Signer { inner }))
    }

    pub(crate) fn run_in_thread(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        let inner = this.inner.clone();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        std::thread::spawn(move || {
            runtime.block_on(async move {
                debug!("Running signer");
                inner.run_forever().await
            })
        });
        Ok(cx.undefined())
    }

    pub(crate) fn run_forever(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        let _ = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(this.inner.run_forever());
        Ok(cx.undefined())
    }

    pub(crate) fn node_id(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<Signer>>(0)?;
        let node_id = this.inner.node_id();
        let buf = JsBuffer::new(&mut cx, 33)?;
        cx.borrow(&buf, |buf| buf.as_mut_slice().copy_from_slice(&node_id));
        Ok(buf)
    }
}

impl Finalize for Signer {}
