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
use super::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use crate::node::{Client, ClnClient};
use crate::pb::{Custommsg, StreamCustommsgRequest};
use cln_grpc::pb::SendcustommsgRequest;
use std::io::{Cursor, Read, Write};

// BOLT8 message ID 37913
const LSPS_MESSAGE_ID: [u8; 2] = [0x94, 0x19];
const TIMEOUT_MILLIS: u128 = 30_000;

pub struct JsonRpcTransport {
    client: Client,        // Used for receiving custom messages
    cln_client: ClnClient, // USed for sending custom message
}

pub enum TransportError {
    JsonParseError(serde_json::Error),
    Other(String),
    GrpcError(tonic::Status),
    ConnectionClosed,
    Timeout,
}

impl From<serde_json::Error> for TransportError {
    fn from(value: serde_json::Error) -> Self {
        return Self::JsonParseError(value);
    }
}

impl From<std::io::Error> for TransportError {
    fn from(value: std::io::Error) -> Self {
        return Self::Other(value.to_string());
    }
}

impl From<tonic::Status> for TransportError {
    fn from(value: tonic::Status) -> Self {
        Self::GrpcError(value)
    }
}

impl JsonRpcTransport {
    pub async fn request<I, O, E>(
        &mut self,
        peer_id: Vec<u8>,
        request: JsonRpcRequest<I>,
    ) -> Result<JsonRpcResponse<O, E>, TransportError>
    where
        I: serde::Serialize,
        E: serde::de::DeserializeOwned,
        O: serde::de::DeserializeOwned,
    {
        // Constructs the message by serializing JsonRpcRequest
        // Core-lightning uses the convention that the first two bytes are the BOLT-8 message id
        let mut cursor: Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        cursor.write(&LSPS_MESSAGE_ID)?;
        serde_json::to_writer(&mut cursor, &request)?;

        let custom_message_request = SendcustommsgRequest {
            node_id: peer_id.clone(),
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
        // It doesn't mean that the LSPS has already responded to it
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
            msg_cursor.read(&mut msg_bolt8_id)?;

            if msg_bolt8_id != LSPS_MESSAGE_ID {
                continue;
            }

            // Deserialize the JSON compare the json_rpc_id
            // If it matches we return a typed JsonRpcRequest
            let value: serde_json::Value = serde_json::from_reader(&mut msg_cursor)?;
            if value.get("id").and_then(|x| x.as_str()) == Some(&request.id) {
                let rpc_response: JsonRpcResponse<O, E> = serde_json::from_value(value)?;
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
                return Err(TransportError::Timeout);
            }
        }

        // If the stream was closed
        return Err(TransportError::ConnectionClosed);
    }
}
