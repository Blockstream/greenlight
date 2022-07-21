use crate::wire::Message;
use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ByteOrder};

tonic::include_proto!("greenlight");

impl HsmRequestContext {
    pub fn from_client_hsmfd_msg(msg: &Message) -> Result<HsmRequestContext> {
        if msg.msgtype() != 9 {
            return Err(anyhow!("message is not an init"));
        }
        let node_id = &msg.body[2..35];
        let dbid = BigEndian::read_u64(&msg.body[35..43]);
        let caps = BigEndian::read_u64(&msg.body[43..51]);
        Ok(HsmRequestContext {
            node_id: node_id.to_vec(),
            dbid: dbid,
            capabilities: caps,
        })
    }
}
