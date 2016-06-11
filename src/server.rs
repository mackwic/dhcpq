use errors::Error;
use mio::*;
use mio::udp::*;
use std::net::SocketAddr;

const SERVER: Token = Token(0);

#[derive(Debug)]
pub struct Server {
    address : SocketAddr,
    socket : UdpSocket,
    event_loop : EventLoop<InnerServer>,
}

impl Server {
    pub fn new(address : SocketAddr) -> Result<Server, Error> { 

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
    
    pub fn run(&mut self) {
        self.socket.bind(&self.address).unwrap();

        let mut inner = InnerServer { tick_counter : 0 };
        self.event_loop.run(&mut inner).unwrap();
    }
}

#[derive(Debug)]
struct InnerServer {
    tick_counter: u32,
}

impl Handler for InnerServer {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<Self>, _token: Token, _events: EventSet) {
        self.tick_counter += 1;
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    mod new {
        pub use super::*;

        #[test]
        fn it_cant_new_with_ipv6() {
            let s = Server::new("[::ff]:6767".parse().unwrap());
            assert!(s.is_err());
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

        #[test]
        fn ready_can_be_called() {
            let mut inner = InnerServer { tick_counter : 0 };
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(1, inner.tick_counter);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(2, inner.tick_counter);
        }
    }
}
