use std::io;

use socket2::SockAddr;
use tokio::prelude::*;
use tokio::reactor::PollEvented2;
use futures::Poll;
use pnet_packet::icmp::Icmp;

use crate::evented::EventedSocket;
use crate::recv::RecvIcmpPacket;
use crate::send::SendIcmpPacket;

#[derive(Debug)]
pub struct Socket {
    socket: PollEvented2<EventedSocket>
}

impl Socket {
    pub fn new() -> io::Result<Self> {
        let socket = EventedSocket::new()?;

        Ok(Socket {
            socket: PollEvented2::new(socket)
        })
    }

    pub fn poll_recv_from(&mut self, buffer: &mut [u8]) -> Poll<(usize, SockAddr), io::Error> {
        try_ready!(self.socket.poll_read_ready(mio::Ready::readable()));

        let recv = self.socket
            .get_ref()
            .recv_from(buffer);

        match recv {
            Ok((n, sender)) => {
                Ok(Async::Ready((n, sender)))
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.socket.clear_read_ready(mio::Ready::readable())?;
                Ok(Async::NotReady)
            },
            Err(e) => Err(e)
        }
    }

    pub fn recv_from(self) -> RecvIcmpPacket {
        RecvIcmpPacket::new(self)
    }

    pub fn poll_send_to(&mut self, buffer: &[u8], host: &SockAddr) -> Poll<usize, io::Error> {
        try_ready!(self.socket.poll_write_ready());

        let send = self.socket
            .get_ref()
            .send_to(buffer, host);

        match send {
            Ok(n) => Ok(Async::Ready(n)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.socket.clear_write_ready()?;
                Ok(Async::NotReady)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn send_to(self, packet: Icmp, host: SockAddr) -> SendIcmpPacket {
        SendIcmpPacket::new(self, packet, host)
    }
}