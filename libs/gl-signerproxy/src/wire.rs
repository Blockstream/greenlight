use crate::passfd::AsyncFdPassingExt;
use anyhow::{Error, Result};
use byteorder::{BigEndian, ByteOrder};
use log::trace;
use std::os::unix::io::RawFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub struct DaemonConnection {
    conn: UnixStream,
}

#[derive(Clone, Debug)]
pub struct Message {
    typ: u16,
    pub body: Vec<u8>,
    pub(crate) fds: Vec<RawFd>,
}

impl Message {
    pub fn from_raw(raw: Vec<u8>) -> Message {
        Message::new(raw)
    }

    pub fn new(raw: Vec<u8>) -> Message {
        Message {
            typ: BigEndian::read_u16(&raw),
            body: raw,
            fds: vec![],
        }
    }

    pub fn msgtype(&self) -> u16 {
        self.typ
    }

    pub fn new_with_fds(raw: Vec<u8>, fds: &[RawFd]) -> Message {
        Message {
            typ: BigEndian::read_u16(&raw),
            body: raw,
            fds: fds.to_vec(),
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && self.typ == other.typ && self.fds == other.fds
    }
}

impl DaemonConnection {
    pub fn new(conn: UnixStream) -> DaemonConnection {
        DaemonConnection { conn }
    }

    fn count_fds(typ: u16) -> i8 {
        match typ {
            109 => 1,
            _ => 0,
        }
    }

    pub async fn read(&mut self) -> Result<Message, Error> {
        let mut len_buf = [0u8; 4];
        self.conn.read_exact(&mut len_buf).await?;
        let msglen = BigEndian::read_u32(&len_buf);

        let mut buf = vec![0u8; msglen as usize];
        self.conn.read_exact(&mut buf).await?;

        let typ = BigEndian::read_u16(&buf);
        let numfds = DaemonConnection::count_fds(typ);
        let mut fds = vec![];
        for _ in 0..numfds {
            fds.push(self.conn.recv_fd().await.map_err(Error::from)?);
        }

        if fds.is_empty() {
            Ok(Message::new(buf))
        } else {
            Ok(Message::new_with_fds(buf, &fds))
        }
    }

    pub async fn write(&mut self, msg: Message) -> Result<(), Error> {
        trace!(
            "Sending message {} ({} bytes, {} FDs)",
            msg.typ,
            msg.body.len(),
            msg.fds.len()
        );

        let mut len_buf = [0u8; 4];
        BigEndian::write_u32(&mut len_buf, msg.body.len() as u32);
        self.conn.write_all(&len_buf).await?;
        self.conn.write_all(&msg.body).await?;

        for fd in msg.fds {
            self.conn.send_fd(fd).await.map_err(Error::from)?;
        }

        Ok(())
    }
}
