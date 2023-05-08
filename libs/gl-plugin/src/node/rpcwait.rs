use tonic::transport::NamedService;
use log::warn;
use tower::Service;

/// The RPC socket will not be available right away, so we wrap the
/// cln-grpc service with this `Service` which essentially checks for
/// the file's existence, and if it doesn't exist we wait for up to 5
/// seconds for it to appear.
#[derive(Debug, Clone)]
pub struct RpcWaitService<S> {
    rpc_path: std::path::PathBuf,
    inner: S,
}

impl<S> RpcWaitService<S> {
    pub fn new(inner: S, rpc_path: std::path::PathBuf) -> Self {
        RpcWaitService { rpc_path, inner }
    }
}

impl<S> Service<hyper::Request<hyper::Body>> for RpcWaitService<S>
where
    S: Service<hyper::Request<hyper::Body>, Response = hyper::Response<tonic::body::BoxBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: hyper::Request<hyper::Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let path = self.rpc_path.clone();
        Box::pin(async move {
            let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
            loop {
                if deadline < tokio::time::Instant::now() {
                    // Break and let it fail in the `inner.call`
                    warn!("Deadline reached, letting the call fail");
                    break;
                }
                match path.metadata() {
                    Ok(_) => break,
                    Err(_) => tokio::time::sleep(tokio::time::Duration::from_millis(500)).await,
                }
            }
            inner.call(request).await
        })
    }
}

impl<S> NamedService for RpcWaitService<S> {
    // Well, this is cheating a bit, since we'll only ever wrap the
    // cln.Node we can have this fixed.
    const NAME: &'static str = "cln.Node";
}
