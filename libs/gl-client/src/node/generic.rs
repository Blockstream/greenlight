//! An implementation of a Grpc Client that does not perform protobuf
//! encoding/decoding. It takes already encoded protobuf messages as
//! `Vec<u8>`, along with the URI and returns the unparsed results to
//! the caller, or a `tonic::Status` in case of failure. This is
//! rather useful when creating bindings, in that only the
//! `GenericClient` and its `call` method need to be mapped through
//! the language boundary, making for a slim interface. This is in
//! contrast to the fat generated interface in which each
//! `tonic::Service` and method on that service is spelled out, and
//! would make for a very wide interface to be mapped.

use bytes::{Buf, BufMut, Bytes};
use http_body::Body;
use log::trace;
use std::str::FromStr;
use tonic::codegen::StdError;

const CODEC: VecCodec = VecCodec {};
const DECODER: VecDecoder = VecDecoder {};
const ENCODER: VecEncoder = VecEncoder {};

/// A GRPC client that can call and return pre-encoded messages. Used
/// by the language bindings to keep the interface between languages
/// small: the client language is used to encode the protobuf
/// payloads, and on the Rust side we just expose the `call` method.
#[derive(Debug, Clone)]
pub struct GenericClient<T> {
    inner: tonic::client::Grpc<T>,
}

impl<T> GenericClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(inner: T) -> Self {
        let inner = tonic::client::Grpc::new(inner);
        Self { inner }
    }

    pub async fn call(
        &mut self,
        path: &str,
        payload: Vec<u8>,
    ) -> Result<tonic::Response<bytes::Bytes>, tonic::Status> {
        trace!(
            "Generic call to {} with {}bytes of payload",
            path,
            payload.len()
        );

        self.inner.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e.into()),
            )
        })?;

        let path = http::uri::PathAndQuery::from_str(path).unwrap();
        self.inner
            .unary(tonic::Request::new(payload), path, CODEC)
            .await
    }

    // TODO Add a `streaming_call` for methods that return a stream to the client
}

/// `tonic::client::Grpc<T>` requires a codec to convert between the
/// in-memory representation (usually protobuf structs generated from
/// IDL) to and from the serialized payload for the call, and the
/// inverse direction for responses. Since the `GenericClient` already
/// takes encoded `Vec<u8>` there is no work for us to do.
#[derive(Default)]
pub struct VecCodec {}

impl Codec for VecCodec {
    type Encode = Vec<u8>;
    type Decode = bytes::Bytes;
    type Encoder = VecEncoder;
    type Decoder = VecDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        ENCODER
    }

    fn decoder(&mut self) -> Self::Decoder {
        DECODER
    }
}

use tonic::codec::{Codec, Decoder, Encoder};

#[derive(Debug, Clone, Default)]
pub struct VecEncoder;

impl Encoder for VecEncoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(
        &mut self,
        item: Self::Item,
        buf: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        buf.put(item.as_slice());
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct VecDecoder;

impl Decoder for VecDecoder {
    type Item = bytes::Bytes;
    type Error = tonic::Status;
    fn decode(
        &mut self,
        buf: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let buf = buf.copy_to_bytes(buf.remaining());
        Ok(Some(buf))
    }
}
