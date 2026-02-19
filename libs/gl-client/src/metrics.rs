use prost::Message;

const HSM_REQUEST_SIGNER_STATE_FIELD_NUMBER: u32 = 4;
const HSM_RESPONSE_SIGNER_STATE_FIELD_NUMBER: u32 = 5;

fn protobuf_varint_len(mut value: usize) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}

fn signer_state_wire_bytes(entries: &[crate::pb::SignerStateEntry], field_number: u32) -> usize {
    let field_key = ((field_number << 3) | 2) as usize; // wire type 2 = length-delimited
    let field_key_len = protobuf_varint_len(field_key);
    entries
        .iter()
        .map(|entry| {
            let entry_len = entry.encoded_len();
            field_key_len + protobuf_varint_len(entry_len) + entry_len
        })
        .sum()
}

pub fn signer_state_request_wire_bytes(entries: &[crate::pb::SignerStateEntry]) -> usize {
    signer_state_wire_bytes(entries, HSM_REQUEST_SIGNER_STATE_FIELD_NUMBER)
}

pub fn signer_state_response_wire_bytes(entries: &[crate::pb::SignerStateEntry]) -> usize {
    signer_state_wire_bytes(entries, HSM_RESPONSE_SIGNER_STATE_FIELD_NUMBER)
}

pub fn savings_percent(full_wire_bytes: usize, diff_wire_bytes: usize) -> usize {
    if full_wire_bytes == 0 {
        return 0;
    }
    full_wire_bytes
        .saturating_sub(diff_wire_bytes)
        .saturating_mul(100)
        / full_wire_bytes
}
