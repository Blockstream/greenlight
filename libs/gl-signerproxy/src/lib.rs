mod hsmproxy;
mod passfd;
mod pb;
mod wire;

use anyhow::Result;

pub struct Proxy {}

impl Proxy {
    pub fn new() -> Proxy {
        Proxy {}
    }

    pub async fn run(&self) -> Result<()> {
        hsmproxy::run().await
    }
}
