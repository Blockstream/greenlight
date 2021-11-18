extern crate libc;
use libc::{c_void, size_t};
use std::ffi::CString;
use std::fmt;
use std::slice;
use std::sync::Mutex;
#[macro_use]
extern crate lazy_static;

extern "C" {
    fn c_init(secret: *const u8, network: *const i8) -> *const u8;
    fn tal_bytelen(ptr: *const c_void) -> size_t;
    fn tal_free(ptr: *const c_void);
    fn c_handle(
        cap: u64,
        dbid: u64,
        peer_id: *const u8,
        peer_id_len: size_t,
        msg: *const u8,
        msglen: size_t,
    ) -> *const u8;
}

lazy_static! {
    // A mutex indicating the Hsmd configuration we initialized
    // last. Guards the internal state of the C library.
    static ref MUX: Mutex<Option<Hsmd>> = Mutex::new(None);
}

#[derive(Debug)]
pub enum Error {
    Generic,
    StringConversion,
    Internal,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Generic => "generic error in hsmd",
                Error::StringConversion => "error converting string to C string",
                Error::Internal => "internal error in hsmd",
            }
        )
    }
}

impl std::error::Error for Error {}

// Type alias to be used whenever we want to handle combinations of
// capabilities. May be replaced with a proper type at some point.
pub type Capabilities = u64;

#[allow(non_snake_case)]
pub mod Capability {
    pub const ECDH: u64 = 1;
    pub const SIGN_GOSSIP: u64 = 2;
    pub const SIGN_ONCHAIN_TX: u64 = 4;
    pub const COMMITMENT_POINT: u64 = 8;
    pub const SIGN_REMOTE_TX: u64 = 16;
    pub const SIGN_CLOSING_TX: u64 = 32;
    pub const SIGN_WILL_FUND_OFFER: u64 = 64;
    pub const MASTER: u64 = 1024;
}

/// A handle to an hsmd. Allows us to create a thread-safe interface
/// on top of the non-thread-safe underlying C library by acquiring a
/// shared lock and re-initializing in case we switched hsmd instance
/// inbetween.
#[derive(Clone)]
pub struct Hsmd {
    secret: Vec<u8>,
    network: String,
}

impl PartialEq for Hsmd {
    fn eq(&self, other: &Self) -> bool {
        self.secret == other.secret && self.network == other.network
    }
}

impl Eq for Hsmd {}

impl Hsmd {
    pub fn new(secret: Vec<u8>, network: &str) -> Hsmd {
        Hsmd {
            secret: secret.clone(),
            network: network.to_string(),
        }
    }

    /// Initialize the hsmd using the parameters enclosed in the
    /// `Hsmd` handle. Useful to retrieve the `init` message.
    pub fn init(&self) -> Result<Vec<u8>, Error> {
        let mut state = MUX.lock().unwrap();
        *state = Some(self.clone());
        return init(self.secret.clone(), &self.network);
    }

    pub fn handle(
        &self,
        capabilities: Capabilities,
        dbid: Option<u64>,
        peer_id: Option<Vec<u8>>,
        message: Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        let mut state = MUX.lock().unwrap();

        // If the hsmd state isn't what we expect anymore we need to
        // re-initialize.
        let need_init = match &*state {
            None => true,
            Some(s) => s != self,
        };

        if need_init {
            init(self.secret.clone(), &self.network)?;
            *state = Some(self.clone());
        }

        handle(capabilities, dbid, peer_id, message)
    }

    pub fn client(&self, capabilities: Capabilities) -> Client {
        Client {
            hsmd: self.clone(),
            caps: capabilities,
            peer_id: None,
            dbid: None,
        }
    }

    pub fn client_with_context(
        &self,
        capabilities: Capabilities,
        dbid: u64,
        peer_id: Vec<u8>,
    ) -> Client {
        Client {
            hsmd: self.clone(),
            caps: capabilities,
            peer_id: Some(peer_id),
            dbid: Some(dbid),
        }
    }
}

/// A handle to a client. A client is an [`Hsmd`] with an associated
/// context which determines how secrets are generated internally.
pub struct Client {
    hsmd: Hsmd,
    caps: Capabilities,
    peer_id: Option<Vec<u8>>,
    dbid: Option<u64>,
}

impl Client {
    pub fn handle(&self, message: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.hsmd
            .handle(self.caps, self.dbid, self.peer_id.clone(), message)
    }
}

/// Call the `hsmd_init` C function.
///
/// This overwrites the internal state of the `libhsmd` library,
/// external synchronization is required if multiple contexts are
/// used. Refer to [`Hsmd`] to see how a thread-safe and/or
/// interleaving interface may look like.
///
/// # Thread-Safety
///
/// This function is part of the non-thread-safe interface together
/// with [`handle`]. If you call `init` with different parameters make
/// sure that `init` and `handle` are not interleaved.
pub fn init(secret: Vec<u8>, network: &str) -> Result<Vec<u8>, Error> {
    let network = match CString::new(network) {
        Ok(s) => s,
        Err(_) => return Err(Error::StringConversion),
    };

    let res: *const u8 = unsafe { c_init(secret.as_ptr(), network.as_ptr()) };

    if res.is_null() {
        return Err(Error::Internal);
    }

    unsafe {
        let reslen = tal_bytelen(res as *const c_void);
        let s = slice::from_raw_parts(res, reslen);
        let response: Vec<u8> = s.clone().to_vec();
        tal_free(res as *const c_void);
        drop(res);
        Ok(response)
    }
}

pub fn handle(
    capabilities: Capabilities,
    dbid: Option<u64>,
    peer_id: Option<Vec<u8>>,
    msg: Vec<u8>,
) -> Result<Vec<u8>, Error> {
    let peer_id = peer_id.unwrap_or_default();

    let res: *const u8 = unsafe {
        c_handle(
            capabilities as u64,
            dbid.unwrap_or_default(),
            peer_id.as_ptr(),
            peer_id.len(),
            msg.as_ptr(),
            msg.len(),
        )
    };
    unsafe {
        let reslen = tal_bytelen(res as *const c_void);
        let s = slice::from_raw_parts(res, reslen);
        let response: Vec<u8> = s.clone().to_vec();
        tal_free(res as *const c_void);
        drop(res);
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    lazy_static! {
        static ref FUNDCHANNEL_REQ: Vec<u8> = hex::decode("00130200000001f25c0c5f21c46ed3f1063a9a41a489ed4e6bb2c18ef1998eb6618198b17137f90000000000666a6c8001e985010000000000160014f4d3100ee3828a602cbc47b1d70ac204e3342081f06a98200000012d70736274ff0100520200000001f25c0c5f21c46ed3f1063a9a41a489ed4e6bb2c18ef1998eb6618198b17137f90000000000666a6c8001e985010000000000160014f4d3100ee3828a602cbc47b1d70ac204e3342081f06a98200001012ba086010000000000220020cc2ef6e3d8102826a2167b463a77bfaf5b57c1ec24c52115d3c471320ad475720105475221026ecfe4d4dd089bbadcf19c860580dae91b2752261269c1f770f32f80e80680492102df1c73d6d45af5edac96953e0eaae5677411b46791092b4bef2c59648b83c20c52ae220602df1c73d6d45af5edac96953e0eaae5677411b46791092b4bef2c59648b83c20c0841c77c75000000002206026ecfe4d4dd089bbadcf19c860580dae91b2752261269c1f770f32f80e806804908109206e600000000000002df1c73d6d45af5edac96953e0eaae5677411b46791092b4bef2c59648b83c20c039c7fa80e43780ad63af6df575924b4c9ae3f1ad22f74067c28227fb45817b2e301").unwrap();

    static ref FUNDCHANNEL_RESP: Vec<u8> = hex::decode("0070141f930e76a8303ac0fe0eaa21cd6afd385453784c79799ed54f10f7a0d8980501e4e30f4208e93d526e7f3cb5592b723db5e56cf764e2540113101518fc7fe501").unwrap();
    }

    #[test]
    fn test_init() {
        let secret = [0 as u8; 32];
        let network = "bitcoin";
        let hsmd = Hsmd::new(secret.to_vec(), network);
        let response = dbg!(hsmd.init()).unwrap();

        let response = dbg!(init(secret.to_vec(), network)).unwrap();
        assert_eq!(response.len(), 177);
    }

    #[test]
    fn test_handle() {
        let secret =
            hex::decode("9f5a9ba98d7e816eebf496db2ff760dc17a4a2f0ae5a87c37cab4bbf6ee05530")
                .unwrap();
        let network = "testnet";

        let msg = FUNDCHANNEL_REQ.to_vec();
        let expected = FUNDCHANNEL_RESP.to_vec();

        let hsmd = Hsmd::new(secret, network);
        let capabilities = Capability::SIGN_REMOTE_TX | Capability::COMMITMENT_POINT;
        let dbid = Some(1);
        let node_id = Some(
            hex::decode("02312627fdf07fbdd7e5ddb136611bdde9b00d26821d14d94891395452f67af248")
                .unwrap(),
        );
        let res = dbg!(hsmd.handle(capabilities, dbid, node_id, msg));
        assert_eq!(res.unwrap(), expected);
    }

    #[test]
    fn test_hsmd_client_handle() {
        let secret =
            hex::decode("9f5a9ba98d7e816eebf496db2ff760dc17a4a2f0ae5a87c37cab4bbf6ee05530")
                .unwrap();
        let network = "testnet";

        let msg = FUNDCHANNEL_REQ.to_vec();
        let expected = FUNDCHANNEL_RESP.to_vec();

        let capabilities = 24;
        let dbid = 1;
        let node_id =
            hex::decode("02312627fdf07fbdd7e5ddb136611bdde9b00d26821d14d94891395452f67af248")
                .unwrap();

        let client = Hsmd::new(secret, network).client_with_context(capabilities, dbid, node_id);
        let res = dbg!(client.handle(msg));
        assert_eq!(res.unwrap(), expected);
    }

    #[test]
    fn test_sign_message() {
        let secret = [0 as u8; 32];
        let network = "bitcoin";
        let hsmd = Hsmd::new(secret.to_vec(), network);

        let cap = 1024;
        let request = hex::decode("0017000B48656c6c6f20776f726c64").unwrap();
        let expected = hex::decode("007b7fab40f2920dedc9ea573fc1a3eefd0492e85b8e4ee6874d57b12526c3a012694c940072eb12c0d263881b909d3c4bb8ea4ec5cc7bff21922689fb0e2e39018501").unwrap();

        let response = dbg!(hsmd.handle(cap, None, None, request)).unwrap();
        assert_eq!(expected, response);
    }

    #[test]
    fn test_capabilities() {
        let caps: Capabilities = Capability::MASTER | Capability::SIGN_GOSSIP | Capability::ECDH;
        assert_eq!(caps, 1027);
    }
}
