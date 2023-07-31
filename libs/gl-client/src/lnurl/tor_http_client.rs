use anyhow::Result;
use arti_client::{TorClient, TorClientConfig};
use url::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct TorHttpClient {
    tor_client: TorClient<tor_rtcompat::PreferredRuntime>,
}

impl TorHttpClient {
    pub async fn new() -> Result<Self> {
        let mut config_builder = TorClientConfig::builder();
        let address_filter = config_builder.address_filter();
        address_filter.allow_onion_addrs(true);
        address_filter.build()?;
        config_builder.build()?;

        let tor_client = TorClient::create_bootstrapped(config_builder.build().unwrap()).await?;
        Ok(Self { tor_client })
    }

    pub async fn tor_http_get(&mut self, url: Url) -> Result<()> {
      println!("Connecting to onion service. This may take a while...");
      let mut stream = self.tor_client.connect((url.domain().unwrap(), 80)).await.unwrap();
    
      println!("Connecting to {}...",  url.to_string());
      let http_message = format!(
          "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
          url.path(),
          url.domain().unwrap()
      )
      .as_bytes()
      .to_owned();
    
      stream.write_all(&http_message).await.unwrap();
      stream.flush().await.unwrap();
    
      let mut buf = Vec::new();
      stream.read_to_end(&mut buf).await?;
      println!("{}", String::from_utf8_lossy(&buf));
      Ok(())
    }
}
