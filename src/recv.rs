use std::io;

use pnet_packet::{FromPacket, Packet};
use pnet_packet::ipv4::Ipv4Packet;
use pnet_packet::icmp::{Icmp, IcmpPacket};
use tokio::prelude::*;
use socket2::SockAddr;

use crate::socket::Socket;

#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct RecvIcmpPacket {
    inner: Option<Socket>
}

impl RecvIcmpPacket {
    pub fn new(socket: Socket) -> Self {
        RecvIcmpPacket { 
            inner: Some(socket)
        }
    }
}

impl Future for RecvIcmpPacket {
    type Item = (Socket, SockAddr, Icmp);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (host, packet) = {
            let ref mut inner = self.inner.as_mut()
                .expect("RecvIcmpPacket polled after completion");

            let mut buffer: [u8; 1024] = unsafe { ::std::mem::uninitialized() };
            let (n, host) = try_ready!(inner.poll_recv_from(&mut buffer));
            let ip = Ipv4Packet::new(&buffer[..n])
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Incomplete read (ip)")
                })?;

            let icmp = IcmpPacket::new(ip.payload())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Incomplete read (icmp)")
                })?;

            (host, icmp.from_packet())
        };

        let socket = self.inner.take().unwrap();
        Ok(Async::Ready((socket, host, packet)))
    }
}