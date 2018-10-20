use std::io;

use tokio::prelude::*;
use pnet_packet::Packet;
use pnet_packet::icmp::{self, Icmp, MutableIcmpPacket};
use socket2::SockAddr;

use crate::socket::Socket;


#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct SendIcmpPacket {
    inner: Option<InnerSend>
}

impl SendIcmpPacket {
    pub fn new(socket: Socket, packet: Icmp, host: SockAddr) -> Self {
        let inner = Some(InnerSend {
            socket,
            packet,
            host
        });

        SendIcmpPacket { inner }
    }
}

#[derive(Debug)]
struct InnerSend {
    socket: Socket,
    packet: Icmp,
    host: SockAddr
}

impl Future for SendIcmpPacket {
    type Item = (Socket, Icmp);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        {
        let ref mut inner = self.inner.as_mut()
            .expect("tokio_icmp::SendIcmpPacket polled after completion");

        let mut buffer: [u8; 1024] = unsafe { ::std::mem::uninitialized() };
        let mut icmp = MutableIcmpPacket::new(&mut buffer).unwrap();
        icmp.populate(&inner.packet);

        let checksum = {
            let immutable = icmp.to_immutable();
            icmp::checksum(&immutable)
        };

        icmp.set_checksum(checksum);

        let n = try_ready!(inner.socket.poll_send_to(icmp.packet(), &inner.host));
        if n != icmp.packet().len() {
            let msg = "Failed to send entire icmp packet";
            let e = io::Error::new(io::ErrorKind::Other, msg);
            return Err(e);
        }
        }

        let inner = self.inner.take().unwrap();
        Ok(Async::Ready((inner.socket, inner.packet)))
    }
}