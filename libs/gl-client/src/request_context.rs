const DOMAIN: &[u8] = b"greenlight/request-context/v1\0";

/// The bytes a client signs and the signer verifies to authorize an RPC.
/// Integers are big-endian.
///
/// A protobuf body can decode as several request types, so the URI is covered
/// to stop the node relabeling a signed request as a different RPC.
pub(crate) fn payload(uri: &str, frame: &[u8], timestamp: u64) -> Vec<u8> {
    [
        DOMAIN,
        &(uri.len() as u64).to_be_bytes(),
        uri.as_bytes(),
        &(frame.len() as u64).to_be_bytes(),
        frame,
        &timestamp.to_be_bytes(),
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_wire_format() {
        // Fixed bytes document the protocol independently of the encoder.
        let expected = b"greenlight/request-context/v1\0\
            \0\0\0\0\0\0\0\x11/cln.Node/Getinfo\
            \0\0\0\0\0\0\0\x05\0\0\0\0\0\
            \x01\x02\x03\x04\x05\x06\x07\x08";
        assert_eq!(
            payload("/cln.Node/Getinfo", &[0; 5], 0x0102030405060708),
            expected
        );
    }
}
