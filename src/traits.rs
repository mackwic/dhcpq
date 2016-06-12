
use std::fmt::Debug;
use std::io;
use std::net;
use mio;

pub trait SocketTrait : Debug {
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<Option<(usize, net::SocketAddr)>>;
}

impl SocketTrait for mio::udp::UdpSocket {
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<Option<(usize, net::SocketAddr)>> {
        self.recv_from(buf)
    }
}
