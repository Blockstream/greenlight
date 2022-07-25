use anyhow::{anyhow, Context, Result};
use log::trace;
use std::net::SocketAddr;
use tonic::transport;

/// Enumeration of supported networks
#[derive(Clone, Debug)]
pub enum Network {
    Bitcoin = 0,
    Testnet = 1,
    Regtest = 2,
}

impl TryFrom<i16> for Network {
    type Error = anyhow::Error;

    fn try_from(i: i16) -> Result<Network> {
        match i {
            0 => Ok(Network::Bitcoin),
            1 => Ok(Network::Testnet),
            2 => Ok(Network::Regtest),
            e => Err(anyhow!("Unknown numeric network {}", e)),
        }
    }
}

impl TryFrom<String> for Network {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Network> {
        match s.to_lowercase().as_ref() {
            "bitcoin" => Ok(Network::Bitcoin),
            "testnet" => Ok(Network::Testnet),
            "regtest" => Ok(Network::Regtest),
            o => Err(anyhow!("Unknown network {}", o)),
        }
    }
}
impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match *self {
                Network::Bitcoin => "bitcoin",
                Network::Testnet => "testnet",
                Network::Regtest => "regtest",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct Identity {
    pub id: transport::Identity,
    pub ca: transport::Certificate,
}

impl Identity {
    /// Loads an identity from the file-system
    ///
    /// Starting from the current directory, we go and look for
    /// certificates and keys for the specified identity. If
    /// `GL_CERT_PATH` is set we instead use that path to look for
    /// the certificates.
    ///
    /// If any of the files we expect to find is missing
    /// (`ca.pem`, `<path>.crt` or `<path>-key.pem`), an error is
    /// returned.
    pub fn from_path(path: &str) -> Result<Identity> {
        let mut dir = std::env::current_dir()?;
        dir.push(std::env::var("GL_CERT_PATH").unwrap_or("./certs/".into()));
        let mut dir = dir.canonicalize().with_context(|| {
            format!(
                "could not canonicalize GL_CERT_PATH={}",
                dir.to_string_lossy()
            )
        })?;

        let cacert = dir.join("ca.pem");
        trace!("Loading root CA from {}", cacert.to_string_lossy());
        /* The root CA cert is at the root of the certs directory. */
        let ca = std::fs::read(cacert).with_context(|| {
            format!(
                "Could not load CA certificate from {}",
                dir.join("ca.pem").to_string_lossy()
            )
        })?;

        /* Find the subdirectory. */
        for p in path.to_string().split("/").skip(1).collect::<Vec<&str>>() {
            dir = dir.join(p);
        }

        let client_key_path = format!("{}-key.pem", dir.to_string_lossy());
        let client_cert_path = format!("{}.crt", dir.to_string_lossy());

        trace!(
            "Loading identity from {} and certificate from {}",
            client_key_path,
            client_cert_path
        );
        let client_cert = std::fs::read(&client_cert_path).with_context(|| {
            format!(
                "could not read client certificate from {:?}",
                client_cert_path
            )
        })?;

        let client_key = std::fs::read(&client_key_path)
            .with_context(|| format!("could not read client key from {:?}", client_key_path))?;

        Ok(Identity {
            id: tonic::transport::Identity::from_pem(client_cert, client_key),
            ca: tonic::transport::Certificate::from_pem(ca),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub identity: Identity,
    pub hsmd_sock_path: String,
    pub node_grpc_binding: String,
    pub node_info: NodeInfo,
    pub towerd_public_grpc_uri: Option<String>,

    /// The `clientca` is the CA we're enforcing when connecting to
    /// other services. This means that services must have a valid
    /// certificate under this CA otherwise the connection is
    /// closed. This is _not_ the CA we use to enforce User
    /// identities. See [`Identity`] for this purpose.
    pub clientca: tonic::transport::Certificate,

    /// The `Nodelet` told us that we're running on this network.
    pub network: Network,
}

impl Config {
    pub fn new() -> Result<Self> {
        let binding: SocketAddr = std::env::var("GL_NODE_BIND")
            .context("Missing GL_NODE_BIND environment variable")?
            .parse()
            .context("Could not parse address from GL_BIND_NODE")?;

        let towerd_public_grpc_uri: Option<String> = std::env::var("GL_TOWER_PUBLIC_GRPC_URI").ok();

        let clientca_path: String = std::env::var("GL_PLUGIN_CLIENTCA_PATH")
            .context("Missing GL_PLUGIN_CLIENTCA_PATH environment variable")?;

        let identity = Identity::from_path(&"/users/1/server")?;

        let clientca = tonic::transport::Certificate::from_pem(std::fs::read(clientca_path)?);
        let network: Network = std::env::var("GL_NODE_NETWORK")
            .context("Missing GL_NODE_NETWORK")?
            .try_into()
            .context("Unknown network in GL_NODE_NETWORK")?;

        Ok(Config {
            identity,
            hsmd_sock_path: "hsmd.sock".to_string(),
            node_grpc_binding: binding.to_string(),
            node_info: NodeInfo::new()?,
            towerd_public_grpc_uri,
            clientca,
            network,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub node_id: Vec<u8>,
    pub initmsg: Vec<u8>,
}

impl NodeInfo {
    fn new() -> Result<Self> {
        let node_id = match std::env::var("GL_NODE_ID") {
            Ok(v) => hex::decode(v)?,
            Err(_) => return Err(anyhow!("Environment variable GL_NODE_ID is not set")),
        };

        let initmsg = hex::decode(std::env::var("GL_NODE_INIT").context("Missing GL_NODE_INIT")?)
            .context("Malformed GL_NODE_INIT")?;

        if node_id.len() != 33 {
            return Err(anyhow!(
                "GL_NODE_ID is not a 33 byte hex-encoded public-key",
            ));
        }

        Ok(NodeInfo {
            node_id: node_id,
            initmsg: initmsg,
        })
    }
}
