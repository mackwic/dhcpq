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
            Ok(Some((count, _))) => self.bytes_read += count,
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
        use std::sync::mpsc;
        use std::net;
        use traits;


        #[derive(Debug)]
        struct UdpSocketMock<'a> {
            receiver: mpsc::Receiver<&'a [u8]>,
        }

        impl<'a> UdpSocketMock<'a> {
            fn new() -> (mpsc::Sender<&'a [u8]>, UdpSocketMock<'a>) {
                let (send, recv) = mpsc::channel();
                (send, UdpSocketMock { receiver: recv })
            }
        }

        impl<'a> traits::SocketTrait for UdpSocketMock<'a> {
            fn recv_from(&self, mut buf: &mut [u8]) -> io::Result<Option<(usize, net::SocketAddr)>> {

                let mut received_values = try!(self.receiver.try_recv()
                                               .or_else(|err| {
                                                   Err(io::Error::new(io::ErrorKind::Other,err))
                                               }));

                let count = try!(io::copy(&mut received_values, &mut buf));
                Ok(Some((count as usize, "0.0.0.0:0".parse().unwrap())))
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
            let (_, sock) = UdpSocketMock::new();
            let inner = InnerServer::new(&sock);
            assert_eq!(0, inner.bytes_read);
        }

        #[test]
        fn it_can_read_from_socket() {
            let (sender, sock) = UdpSocketMock::new();
            let data = "hello";
            let len  = data.len();
            sender.send(data.as_bytes()).unwrap();

            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(len, inner.bytes_read);
        }

        #[test]
        fn bytes_read_is_the_same_as_input_size() {
            let (sender, sock) = UdpSocketMock::new();
            let data = "hello hello";
            let len  = data.len();
            sender.send(data.as_bytes()).unwrap();

            let mut inner = InnerServer::new(&sock);
            inner.ready(&mut EventLoop::new().unwrap(), SERVER, EventSet::all());
            assert_eq!(len, inner.bytes_read);
        }

        #[test]
        fn bytes_read_increase_each_time_something_is_read() {
            let (sender, sock) = UdpSocketMock::new();

            let mut ev = EventLoop::new().unwrap();
            let mut inner = InnerServer::new(&sock);
            let data1 = "hello hello";
            let data2 = "g0t r00t";

            sender.send(data1.as_bytes()).unwrap();
            inner.ready(&mut ev, SERVER, EventSet::all());
            assert_eq!(data1.len(), inner.bytes_read);

            sender.send(data2.as_bytes()).unwrap();
            inner.ready(&mut ev, SERVER, EventSet::all());
            assert_eq!(data1.len() + data2.len(), inner.bytes_read);
        }
    }
}
