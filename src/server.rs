use errors::Error;
use mio::*;
use mio::udp::*;
use std::net::SocketAddr;
use traits::*;

const SERVER: Token = Token(0);

#[derive(Debug)]
pub struct Server<'a> {
    address : SocketAddr,
    socket : UdpSocket,
    event_loop : EventLoop<InnerServer<'a, UdpSocket>>,
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
struct InnerServer<'a, S> where S : 'a + SocketTrait {

    tick_counter:   usize,
    bytes_read:     usize,
    socket:         &'a S,
}

impl<'a, Sock : SocketTrait> InnerServer<'a, Sock> {
    fn new<'serv_instance, 'server>(socket: &'server Sock) -> InnerServer<'serv_instance, Sock>
        where 'server : 'serv_instance {

        InnerServer {
            tick_counter: 0,
            bytes_read: 0,
            socket: socket,
        }
    }
}

impl<'a, Sock : SocketTrait> Handler for InnerServer<'a, Sock> {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<Self>, _token: Token, _events: EventSet) {
        self.tick_counter += 1;
        let mut buffer : [u8; 4096] = [0; 4096];

        match self.socket.recv_from(&mut buffer) {
            Ok(Some((count, _))) => self.bytes_read = count,
            _ => ()
        }
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
        use std::io;
        use std::net;
        use traits;

        #[derive(Debug, Default)]
        struct UdpSocketMock {
            data: Vec<u8>
        }

        impl UdpSocketMock {
            fn push_data(&mut self, data: Vec<u8>) {
                self.data = data
            }
        }

        impl traits::SocketTrait for UdpSocketMock {
            fn recv_from(&self, _buf: &mut [u8]) -> io::Result<Option<(usize, net::SocketAddr)>> {
                let data = &self.data;
                Ok(Some((data.len(), "0.0.0.0:0".parse().unwrap())))
            }
        }

        #[test]
        fn ready_can_be_called() {
            let sock = UdpSocket::v4().unwrap();
            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(1, inner.tick_counter);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(2, inner.tick_counter);
        }

        #[test]
        fn no_bytes_read_at_initialization() {
            let sock = UdpSocketMock::default();
            let inner = InnerServer::new(&sock);
            assert_eq!(0, inner.bytes_read);
        }

        #[test]
        fn it_can_read_from_socket() {
            let mut sock = UdpSocketMock::default();
            let data = "hello";
            let len  = data.len();
            sock.push_data(Vec::from(data));

            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(len, inner.bytes_read);
        }

        #[test]
        fn bytes_read_is_the_same_as_input_size() {
            let mut sock = UdpSocketMock::default();
            let data = "hello hello";
            let len  = data.len();
            sock.push_data(Vec::from(data));

            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(len, inner.bytes_read);
        }

        #[test]
        fn bytes_read_increase_each_time_something_is_read() {
            let mut sock = UdpSocketMock::default();
            let data1 = "hello hello";
            let len1  = data1.len();
            sock.push_data(Vec::from(data1));

            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(len1, inner.bytes_read);

            let data2 = "g0t r00t";
            let len2  = data2.len();
            sock.push_data(Vec::from(data2));
            assert_eq!(len1 + len2, inner.bytes_read);
        }
    }
}
