use std::io;
use std::os::unix::io::AsRawFd;

use mio::unix::EventedFd;
use socket2::{Domain, Protocol, Socket as RawSocket, Type, SockAddr};

#[derive(Debug)]
pub(crate) struct EventedSocket {
    inner: RawSocket,
}

impl EventedSocket {
    pub fn new() -> io::Result<Self> {
        let inner = RawSocket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))?;
        inner.set_nonblocking(true)?;

        Ok(EventedSocket { inner })
    }

    pub fn recv_from(&self, buffer: &mut [u8]) -> io::Result<(usize, SockAddr)> {
        self.inner.recv_from(buffer)
    }

    pub fn send_to(&self, buffer: &[u8], host: &SockAddr) -> io::Result<usize> {
        self.inner.send_to(buffer, host)
    }
}

impl mio::event::Evented for EventedSocket {
    fn register(
        &self,
        poll: &mio::Poll,
        token: mio::Token,
        interest: mio::Ready,
        opts: mio::PollOpt,
    ) -> io::Result<()> {
        let fd = self.inner.as_raw_fd();
        let evented_fd = EventedFd(&fd);
        evented_fd.register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &mio::Poll,
        token: mio::Token,
        interest: mio::Ready,
        opts: mio::PollOpt,
    ) -> io::Result<()> {
        let fd = self.inner.as_raw_fd();
        let evented_fd = EventedFd(&fd);
        evented_fd.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
        let fd = self.inner.as_raw_fd();
        let evented_fd = EventedFd(&fd);
        evented_fd.deregister(poll)
    }
}