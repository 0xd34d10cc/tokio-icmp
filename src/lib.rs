#[macro_use]
extern crate futures;

mod evented;
mod socket;
mod send;
mod recv;

pub use crate::socket::Socket;