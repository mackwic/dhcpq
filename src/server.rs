use errors::Error;
use mio::*;
use mio::udp::*;
use std::net::SocketAddr;

const SERVER: Token = Token(0);

#[derive(Debug)]
pub struct Server<'a> {
    address : SocketAddr,
    socket : UdpSocket,
    event_loop : EventLoop<InnerServer<'a>>,
}

impl<'a> Server<'a> {
    pub fn new<'b>(address : SocketAddr) -> Result<Server<'b>, Error> {

        if let SocketAddr::V6(_) = address {
            return Err(Error::Ipv6Unsupported)
        }

        let socket = try!(UdpSocket::v4());
        let mut event_loop = try!(EventLoop::new());
        try!(event_loop.register(&socket, SERVER, EventSet::all(), PollOpt::edge()));

        Ok(Server {
            address : address,
            socket : socket,
            event_loop : event_loop,
        })
    }

    pub fn run<'b>(&'a mut self) -> Result<(), Error> {
        try!(self.socket.bind(&self.address));

        let mut inner = InnerServer::new(&self.socket);
        try!(self.event_loop.run(&mut inner));

        unreachable!()
    }
}

#[derive(Debug)]
struct InnerServer<'a> {
    tick_counter: u32,
    socket: &'a UdpSocket,
}

impl<'a> InnerServer<'a> {
    fn new<'serv_instance, 'server>(socket: &'server UdpSocket) -> InnerServer<'serv_instance>
        where 'server : 'serv_instance {

        InnerServer {
            tick_counter: 0,
            socket: &socket
        }
    }
}

impl<'a> Handler for InnerServer<'a> {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<Self>, _token: Token, _events: EventSet) {
        self.tick_counter += 1;
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    mod server {
        pub use super::*;

        #[test]
        fn it_cant_new_with_ipv6() {
            let s = Server::new("[::ff]:6767".parse().unwrap());
            assert!(s.is_err())
        }

        #[test]
        fn it_can_new_with_ipv4() {
            assert!(Server::new("0.0.0.0:6567".parse().unwrap()).is_ok())
        }
    }

    mod inner_server{
        pub use super::*;
        use super::super::InnerServer;
        use super::super::SERVER;
        use mio::*;
        use mio::udp::UdpSocket;

        #[test]
        fn ready_can_be_called() {
            let sock = UdpSocket::v4().unwrap();
            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(1, inner.tick_counter);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(2, inner.tick_counter);
        }
    }
}
