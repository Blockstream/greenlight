use crate::passfd::SyncFdPassingExt;
use anyhow::{anyhow, Error, Result};
use byteorder::{BigEndian, ByteOrder};
use log::trace;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

/// A simple implementation of the inter-daemon protocol wrapping a
/// UnixStream. Easy to read from and write to.
pub struct DaemonConnection {
    conn: Mutex<UnixStream>,
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
    pub fn new(connection: UnixStream) -> DaemonConnection {
        DaemonConnection {
            conn: Mutex::new(connection),
        }
    }

    fn count_fds(typ: u16) -> i8 {
        match typ {
            109 => 1,
            _ => 0,
        }
    }

    pub fn read(&self) -> Result<Message, Error> {
        let mut sock = self.conn.lock().unwrap();

        // Read 4-byte length prefix in big-endian
        let mut len_buf = [0u8; 4];
        sock.read_exact(&mut len_buf)?;
        let msglen = BigEndian::read_u32(&len_buf);

        // Read the message body
        let mut buf = vec![0u8; msglen as usize];
        sock.read_exact(&mut buf)?;

        if buf.len() < msglen as usize {
            return Err(anyhow!("Short read from client"));
        }

        let typ = BigEndian::read_u16(&buf);
        let mut fds = vec![];

        // Receive any file descriptors associated with this message type
        let numfds = DaemonConnection::count_fds(typ);
        for _ in 0..numfds {
            fds.push(sock.as_raw_fd().recv_fd()?);
        }

        if fds.len() == 0 {
            Ok(Message::new(buf))
        } else {
            Ok(Message::new_with_fds(buf, &fds))
        }
    }

    pub fn write(&self, msg: Message) -> Result<(), Error> {
        trace!(
            "Sending message {} ({} bytes, {} FDs)",
            msg.typ,
            msg.body.len(),
            msg.fds.len()
        );
        let mut client = self.conn.lock().unwrap();

        // Write 4-byte length prefix in big-endian
        let mut len_buf = [0u8; 4];
        BigEndian::write_u32(&mut len_buf, msg.body.len() as u32);
        client.write_all(&len_buf)?;

        // Write the message body
        client.write_all(&msg.body)?;

        // Send any file descriptors
        for fd in msg.fds {
            client.as_raw_fd().send_fd(fd)?;
        }

        Ok(())
    }
}
