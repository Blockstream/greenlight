//
// This is an implementation of the LSPS1 specification (https://github.com/BitcoinAndLightningLayerSpecs/lsp/blob/main/OpenAPI/LSPS1.json)
//
// This specification describes how a lightning-node should communicate with
// a Lightning Service Provider.
//
// The key-idea is that all communication occurs using JSON-rpc and BOLT8-messages
// are used as the transport layer. All messages related to LSPS will start
// with the LSPS_MESSAGE_ID.
//
use super::json_rpc::{generate_random_rpc_id, JsonRpcResponse};
use super::json_rpc_erased::JsonRpcMethodUnerased;
use crate::lsps::error::LspsError;
use crate::node::{Client, ClnClient};
use crate::pb::{Custommsg, StreamCustommsgRequest};
use crate::util::is_feature_bit_enabled;
use cln_grpc::pb::{ListnodesRequest, SendcustommsgRequest};
use std::io::{Cursor, Read, Write};

// BOLT8 message ID 37913
const LSPS_MESSAGE_ID: [u8; 2] = [0x94, 0x19];
const TIMEOUT_MILLIS: u128 = 30_000;
const BITNUM_LSP_FEATURE: usize = 729;

pub struct LspClient {
    client: Client,        // Used for receiving custom messages
    cln_client: ClnClient, // USed for sending custom message
}

impl LspClient {
    pub fn new(client: Client, cln_client: ClnClient) -> Self {
        Self {
						client,
            cln_client,
        }
    }

    /// Create a JSON-rpc request to a LSPS
    ///
    /// # Arguments:
    /// - peer_id: the node_id of the lsps
    /// - method: the request method as defined in the LSPS
    /// - param: the request parameter
    ///
    pub async fn request<'a, I, O, E>(
        &mut self,
        peer_id: &[u8],
        method: &impl JsonRpcMethodUnerased<'a, I, O, E>,
        param: I,
    ) -> Result<JsonRpcResponse<O, E>, LspsError>
    where
        I: serde::Serialize,
        E: serde::de::DeserializeOwned,
        O: serde::de::DeserializeOwned,
    {
        let json_rpc_id = generate_random_rpc_id();
        self
            .request_with_json_rpc_id(peer_id, method, param, json_rpc_id)
            .await
    }

    /// Makes the jsonrpc request and returns the response
    ///
    /// This method allows the user to specify a custom json_rpc_id.
    /// To ensure compliance with LSPS0 the use of `[request`] is recommended.
    /// For some testing scenario's it is useful to have a reproducable json_rpc_id
    pub async fn request_with_json_rpc_id<'a, I, O, E>(
        &mut self,
        peer_id: &[u8],
        method: &impl JsonRpcMethodUnerased<'a, I, O, E>,
        param: I,
        json_rpc_id: String,
    ) -> Result<JsonRpcResponse<O, E>, LspsError>
    where
        I: serde::Serialize,
        E: serde::de::DeserializeOwned,
        O: serde::de::DeserializeOwned,
    {
        // Constructs the JsonRpcRequest
        let request = method
            .create_request(param, json_rpc_id)
            .map_err(LspsError::JsonParseRequestError)?;

        // Core-lightning uses the convention that the first two bytes are the BOLT-8 message id
        let mut cursor: Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        cursor.write_all(&LSPS_MESSAGE_ID)?;
        serde_json::to_writer(&mut cursor, &request)
            .map_err(LspsError::JsonParseRequestError)?;

        let custom_message_request = SendcustommsgRequest {
            node_id: peer_id.to_vec(),
            msg: cursor.into_inner(),
        };

        // Here we start listening to the responses.
        // It is important that we do this before we send the first message
        // to prevent high-latency clients from missing the response
        //
        // The await ensures that we are ready to get incoming messages
        let mut stream: tonic::Streaming<Custommsg> = self
            .client
            .stream_custommsg(StreamCustommsgRequest {})
            .await?
            .into_inner();

        // Sends the JsonRpcRequest
        // Once the await has completed our greenlight node has successfully send the message
        // The LSPS did probably not respond yet
        self.cln_client
            .send_custom_msg(custom_message_request)
            .await?;

        // We read all incoming messages one-by-one
        // If the peer_id, LSPS_MESSAGE_ID and the id used in json_rpc matches we return the result
        let loop_start_instant = std::time::Instant::now();
        while let Some(mut msg) = stream.message().await? {
            // Skip if peer_id doesn't match
            if msg.peer_id != peer_id {
                continue;
            }

            // Skip if LSPS_MESSAGE_ID (first 2 bytes) doesn't match
            let mut msg_cursor: Cursor<&mut [u8]> = std::io::Cursor::new(msg.payload.as_mut());
            let mut msg_bolt8_id: [u8; 2] = [0, 0];
            msg_cursor.read_exact(&mut msg_bolt8_id)?;

            if msg_bolt8_id != LSPS_MESSAGE_ID {
                continue;
            }

            // Deserialize the JSON compare the json_rpc_id
            // If it matches we return a typed JsonRpcRequest
            let value: serde_json::Value = serde_json::from_reader(&mut msg_cursor)
                .map_err(LspsError::JsonParseResponseError)?;
            if value.get("id").and_then(|x| x.as_str()) == Some(&request.id) {
                // There is a bug here. We need to do the parsing in the underlying trait
                let rpc_response = method
                    .parse_json_response_value(value)
                    .map_err(LspsError::JsonParseResponseError)?;
                return Ok(rpc_response);
            }

            // Check if the connection timed-out
            // TODO:
            // This is implementation is somewhat flawed.
            // If no message comes in it will wait forever.
            //
            // I might have to add a built-in time-out mechanism in StreamCustomMsg or come up with
            // a better solution.
            if loop_start_instant.elapsed().as_millis() >= TIMEOUT_MILLIS {
                return Err(LspsError::Timeout);
            }
        }

        // If the stream was closed
        Err(LspsError::ConnectionClosed)
    }

    pub async fn list_lsp_servers(&mut self) -> Result<Vec<Vec<u8>>, LspsError> {
        let request = ListnodesRequest { id: None };

        // Query all known lightning-nodes
        let response = self.cln_client.list_nodes(request).await?;

        return Ok(response
            .into_inner()
            .nodes
            .into_iter()
            .filter(|x| is_feature_bit_enabled(x.features(), BITNUM_LSP_FEATURE))
            .map(|n| n.nodeid)
            .collect());
    }
}
