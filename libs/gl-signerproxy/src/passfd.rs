use libc::{self, c_int, c_uchar, c_void, msghdr};
use std::mem;
use log::trace;
use std::io::{Error, ErrorKind};
use std::os::unix::io::RawFd;

pub trait SyncFdPassingExt {
    /// Send RawFd. No type information is transmitted.
    fn send_fd(&self, fd: RawFd) -> Result<(), Error>;
    /// Receive RawFd. No type information is transmitted.
    fn recv_fd(&self) -> Result<RawFd, Error>;
}
impl SyncFdPassingExt for RawFd {
    fn send_fd(&self, fd: RawFd) -> Result<(), Error> {
        trace!("Sending fd {}", fd);
        let mut dummy: c_uchar = 0;
        let msg_len = unsafe { libc::CMSG_SPACE(mem::size_of::<c_int>() as u32) as _ };
        let mut buf = vec![0u8; msg_len as usize];
        let mut iov = libc::iovec {
            iov_base: &mut dummy as *mut c_uchar as *mut c_void,
            iov_len: mem::size_of_val(&dummy),
        };
        unsafe {
            let hdr = libc::cmsghdr {
                cmsg_level: libc::SOL_SOCKET,
                cmsg_type: libc::SCM_RIGHTS,
                cmsg_len: libc::CMSG_LEN(mem::size_of::<c_int>() as u32) as _,
            };
            // https://github.com/rust-lang/rust-clippy/issues/2881
            #[allow(clippy::cast_ptr_alignment)]
            std::ptr::write_unaligned(buf.as_mut_ptr() as *mut _, hdr);

            // https://github.com/rust-lang/rust-clippy/issues/2881
            #[allow(clippy::cast_ptr_alignment)]
            std::ptr::write_unaligned(
                libc::CMSG_DATA(buf.as_mut_ptr() as *const _) as *mut c_int,
                fd,
            );
        }
        let msg: msghdr = libc::msghdr {
            msg_name: std::ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov,
            msg_iovlen: 1,
            msg_control: buf.as_mut_ptr() as *mut c_void,
            msg_controllen: msg_len,
            msg_flags: 0,
        };

        let rv = unsafe { libc::sendmsg(*self, &msg, 0) };
        if rv < 0 {
            return Err(Error::last_os_error());
        }

        Ok(())
    }

    fn recv_fd(&self) -> Result<RawFd, Error> {
        trace!("Receiving fd");
        let mut dummy: c_uchar = 0;
        let msg_len = unsafe { libc::CMSG_SPACE(mem::size_of::<c_int>() as u32) as _ };
        let mut buf = vec![0u8; msg_len as usize];
        let mut iov = libc::iovec {
            iov_base: &mut dummy as *mut c_uchar as *mut c_void,
            iov_len: mem::size_of_val(&dummy),
        };
        let mut msg: msghdr = libc::msghdr {
            msg_name: std::ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov,
            msg_iovlen: 1,
            msg_control: buf.as_mut_ptr() as *mut c_void,
            msg_controllen: msg_len,
            msg_flags: 0,
        };

        unsafe {
            let rv = libc::recvmsg(*self, &mut msg, 0);
            match rv {
                0 => Err(Error::new(ErrorKind::UnexpectedEof, "0 bytes read")),
                rv if rv < 0 => Err(Error::last_os_error()),
                rv if rv == mem::size_of::<c_uchar>() as isize => {
                    let hdr: *mut libc::cmsghdr =
                        if msg.msg_controllen >= mem::size_of::<libc::cmsghdr>() as _ {
                            msg.msg_control as *mut libc::cmsghdr
                        } else {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "bad control msg (header)",
                            ));
                        };
                    if (*hdr).cmsg_level != libc::SOL_SOCKET || (*hdr).cmsg_type != libc::SCM_RIGHTS
                    {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "bad control msg (level)",
                        ));
                    }
                    if msg.msg_controllen
                        != libc::CMSG_SPACE(mem::size_of::<c_int>() as u32) as usize
                    {
                        return Err(Error::new(ErrorKind::InvalidData, "bad control msg (len)"));
                    }
                    // https://github.com/rust-lang/rust-clippy/issues/2881
                    #[allow(clippy::cast_ptr_alignment)]
                    let fd = std::ptr::read_unaligned(libc::CMSG_DATA(hdr) as *mut c_int);
                    if libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) < 0 {
                        return Err(Error::last_os_error());
                    }
                    Ok(fd)
                }
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    "bad control msg (ret code)",
                )),
            }
        }
    }
}
