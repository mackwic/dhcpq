
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate mio;
#[macro_use(quick_error)]
extern crate quick_error;

use mio::udp::*;
use mio::*;
use std::io;
use std::net::SocketAddr;

////////////////////////////////////////

const IPV4_PORT : u32 = 67;
const SERVER: mio::Token = mio::Token(0);

////////////////////////////////////////

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Ipv6Unsupported {
            description("IPv6 addresses are not supported")
        }
        InnerIo(err: io::Error) {
            from()
        }
    }
}


#[derive(Debug)]
pub struct Server {
    address : SocketAddr,
    socket : UdpSocket,
    event_loop : EventLoop<InnerServer>,
}

#[derive(Debug)]
pub struct InnerServer;

impl Server {
    pub fn new(address : SocketAddr) -> Result<Server, Error> { 

        if let SocketAddr::V6(_) = address {
            return Err(Error::Ipv6Unsupported)
        }

        let socket = try!(UdpSocket::v4());
        let mut event_loop = try!(mio::EventLoop::new());
        try!(event_loop.register(&socket, SERVER, EventSet::all(), PollOpt::edge()));

        Ok(Server {
            address : address,
            socket : socket,
            event_loop : event_loop,
        })
    }
    
    pub fn run(&mut self) {
        self.socket.bind(&self.address).unwrap();

        let mut inner = InnerServer {};
        self.event_loop.run(&mut inner).unwrap();
    }
}

impl mio::Handler for InnerServer {
    type Timeout = ();
    type Message = ();
}

////////////////////////////////////////

fn main() {
    let address = "0.0.0.0:6567".parse().unwrap();

    println!("yo ! {:?}", Server::new(address))
}

////////////////////////////////////////

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
}
